//! Request identity, authorization, and rate-limiting primitives.
//!
//! These controls live above the transport layer so the same policy is applied
//! to STDIO, HTTP, WebSocket, and custom transports.

use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::core::error::{McpError, McpResult};

/// Authenticated identity attached to an MCP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub id: String,
    pub roles: BTreeSet<String>,
    pub attributes: BTreeMap<String, String>,
    pub authentication_method: Option<String>,
}

impl Principal {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            roles: BTreeSet::new(),
            attributes: BTreeMap::new(),
            authentication_method: None,
        }
    }

    pub fn anonymous() -> Self {
        Self::new("anonymous")
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.insert(role.into());
        self
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    pub fn with_authentication_method(mut self, method: impl Into<String>) -> Self {
        self.authentication_method = Some(method.into());
        self
    }
}

/// Context shared by validation, policy, observability, and request handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestContext {
    pub request_id: String,
    pub principal: Principal,
    pub transport: String,
    pub peer_address: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

impl RequestContext {
    pub fn new(principal: Principal) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            principal,
            transport: "unknown".to_string(),
            peer_address: None,
            metadata: BTreeMap::new(),
        }
    }

    pub fn anonymous() -> Self {
        Self::new(Principal::anonymous())
    }

    pub fn with_transport(mut self, transport: impl Into<String>) -> Self {
        self.transport = transport.into();
        self
    }

    pub fn with_peer_address(mut self, address: impl Into<String>) -> Self {
        self.peer_address = Some(address.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Normalized target used by authorization policies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestTarget {
    pub method: String,
    pub resource: Option<String>,
}

impl RequestTarget {
    pub fn new(method: impl Into<String>, resource: Option<String>) -> Self {
        Self {
            method: method.into(),
            resource,
        }
    }
}

/// Authorization decision provider.
#[async_trait]
pub trait Authorizer: Send + Sync {
    async fn authorize(&self, context: &RequestContext, target: &RequestTarget) -> McpResult<()>;
}

/// Backwards-compatible authorizer used unless an application installs RBAC.
#[derive(Debug, Default)]
pub struct AllowAllAuthorizer;

#[async_trait]
impl Authorizer for AllowAllAuthorizer {
    async fn authorize(&self, _context: &RequestContext, _target: &RequestTarget) -> McpResult<()> {
        Ok(())
    }
}

/// One fine-grained role permission. Patterns support exact values or a final
/// `*`, for example `tools/*` and `urn:customer:*`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permission {
    pub role: String,
    pub method_pattern: String,
    pub resource_pattern: Option<String>,
}

impl Permission {
    pub fn new(role: impl Into<String>, method_pattern: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            method_pattern: method_pattern.into(),
            resource_pattern: None,
        }
    }

    pub fn for_resource(mut self, resource_pattern: impl Into<String>) -> Self {
        self.resource_pattern = Some(resource_pattern.into());
        self
    }
}

/// Deny-by-default role-based authorizer.
#[derive(Debug, Clone, Default)]
pub struct RbacAuthorizer {
    permissions: Vec<Permission>,
}

impl RbacAuthorizer {
    pub fn new(permissions: impl IntoIterator<Item = Permission>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
        }
    }

    pub fn allow(mut self, permission: Permission) -> Self {
        self.permissions.push(permission);
        self
    }
}

fn pattern_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    pattern
        .strip_suffix('*')
        .map_or(pattern == value, |prefix| value.starts_with(prefix))
}

#[async_trait]
impl Authorizer for RbacAuthorizer {
    async fn authorize(&self, context: &RequestContext, target: &RequestTarget) -> McpResult<()> {
        let allowed = self.permissions.iter().any(|permission| {
            context.principal.roles.contains(&permission.role)
                && pattern_matches(&permission.method_pattern, &target.method)
                && match (&permission.resource_pattern, &target.resource) {
                    (None, _) => true,
                    (Some(pattern), Some(resource)) => pattern_matches(pattern, resource),
                    (Some(_), None) => false,
                }
        });

        if allowed {
            Ok(())
        } else {
            Err(McpError::Forbidden(format!(
                "principal '{}' cannot access method '{}'{}",
                context.principal.id,
                target.method,
                target
                    .resource
                    .as_ref()
                    .map(|resource| format!(" resource '{resource}'"))
                    .unwrap_or_default()
            )))
        }
    }
}

/// Token-bucket limit applied independently to each principal and method.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub burst: u32,
    pub requests_per_second: f64,
    pub idle_entry_ttl: Duration,
}

impl RateLimitConfig {
    pub fn new(burst: u32, requests_per_second: f64) -> McpResult<Self> {
        if burst == 0 || !requests_per_second.is_finite() || requests_per_second <= 0.0 {
            return Err(McpError::Validation(
                "rate limit requires burst > 0 and requests_per_second > 0".to_string(),
            ));
        }
        Ok(Self {
            burst,
            requests_per_second,
            idle_entry_ttl: Duration::from_secs(600),
        })
    }
}

#[derive(Debug)]
struct Bucket {
    tokens: f64,
    updated_at: Instant,
}

/// Concurrent in-process token-bucket limiter.
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: DashMap<String, Bucket>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: DashMap::new(),
        }
    }

    pub fn check(&self, context: &RequestContext, target: &RequestTarget) -> McpResult<()> {
        let now = Instant::now();
        let key = format!("{}\u{1f}{}", context.principal.id, target.method);
        let mut bucket = self.buckets.entry(key).or_insert_with(|| Bucket {
            tokens: f64::from(self.config.burst),
            updated_at: now,
        });

        let elapsed = now.duration_since(bucket.updated_at).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.config.requests_per_second)
            .min(f64::from(self.config.burst));
        bucket.updated_at = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(())
        } else {
            let retry_after = (1.0 - bucket.tokens) / self.config.requests_per_second;
            Err(McpError::RateLimited {
                retry_after_ms: (retry_after * 1000.0).ceil() as u64,
            })
        }
    }

    /// Removes stale principal/method buckets. Applications may call this from
    /// an existing maintenance loop; request processing never scans the map.
    pub fn prune_idle(&self) {
        let now = Instant::now();
        self.buckets
            .retain(|_, bucket| now.duration_since(bucket.updated_at) < self.config.idle_entry_ttl);
    }
}

/// Shared server request policy.
#[derive(Clone)]
pub struct RequestPolicy {
    authorizer: Arc<dyn Authorizer>,
    rate_limiter: Option<Arc<RateLimiter>>,
}

impl Default for RequestPolicy {
    fn default() -> Self {
        Self {
            authorizer: Arc::new(AllowAllAuthorizer),
            rate_limiter: None,
        }
    }
}

impl RequestPolicy {
    pub fn new(authorizer: impl Authorizer + 'static) -> Self {
        Self {
            authorizer: Arc::new(authorizer),
            rate_limiter: None,
        }
    }

    pub fn with_rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.rate_limiter = Some(Arc::new(limiter));
        self
    }

    pub async fn enforce(&self, context: &RequestContext, target: &RequestTarget) -> McpResult<()> {
        self.authorizer.authorize(context, target).await?;
        if let Some(limiter) = &self.rate_limiter {
            limiter.check(context, target)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rbac_is_deny_by_default_and_checks_resources() {
        let policy = RequestPolicy::new(RbacAuthorizer::new([Permission::new(
            "reader",
            "resources/read",
        )
        .for_resource("urn:public:*")]));
        let context = RequestContext::new(Principal::new("alice").with_role("reader"));

        assert!(policy
            .enforce(
                &context,
                &RequestTarget::new("resources/read", Some("urn:public:1".to_string()))
            )
            .await
            .is_ok());
        assert!(matches!(
            policy
                .enforce(
                    &context,
                    &RequestTarget::new("resources/read", Some("urn:private:1".to_string()))
                )
                .await,
            Err(McpError::Forbidden(_))
        ));
    }

    #[tokio::test]
    async fn rate_limit_is_enforced_per_principal_and_method() {
        let limiter = RateLimiter::new(RateLimitConfig::new(1, 0.01).unwrap());
        let policy = RequestPolicy::default().with_rate_limiter(limiter);
        let context = RequestContext::new(Principal::new("alice"));
        let target = RequestTarget::new("tools/list", None);

        assert!(policy.enforce(&context, &target).await.is_ok());
        assert!(matches!(
            policy.enforce(&context, &target).await,
            Err(McpError::RateLimited { .. })
        ));
        assert!(policy
            .enforce(&context, &RequestTarget::new("ping", None))
            .await
            .is_ok());
    }
}
