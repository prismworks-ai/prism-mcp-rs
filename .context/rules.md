# Project Rules & Guidelines

## Core Principles

* **Maintain a Single Source of Truth**: All information should reside in a single location to prevent duplication. Don't repeat; reference instead.
* **Be Direct and Factual**: Use clear, concise language without commentary or unnecessary details.
* **Stay Current**: Always work from the most recent project state and update tasks atomically as work progresses.

## Product & UX Perspective

* **Validate Business Impact**: Ensure all features align with business objectives and key performance metrics.
* **Focus on User Value**: Always consider the impact on conversion, engagement, and overall user experience.
* **Adhere to Design Standards**: Follow established design patterns and accessibility guidelines (WCAG compliance).

## Technical Execution

* **System Design**: Create and evolve the system architecture, adhering to established service patterns, and ensuring scalability, security, and performance.
* **Development**: Write clean, reusable, and well-documented code that complies with the established tech stack and architectural patterns.
* **Testing & Quality Assurance**: Implement comprehensive testing to ensure the system is robust, performant, and secure. This includes validating model accuracy and addressing potential biases.
* **Deployment & Operations**: Plan for and manage the deployment process, including CI/CD pipelines, monitoring, and logging. Ensure the system is observable and resilient.

## Cross-Functional Coordination

* **Pre-Implementation**: Validate project constraints and dependencies across all roles, identifying and documenting any blockers or missing prerequisites.
* **During Implementation**: Update task status and document any new blockers in real time. Coordinate handoffs between roles by updating relevant sections.
* **Post-Implementation**: Mark tasks as complete, update the feature's status, and document any lessons learned for future retrospectives.

## Code Standards

### Naming Conventions
- Variables: camelCase or snake_case (specify which)
- Functions: descriptive action verbs
- Classes: PascalCase with clear nouns
- Constants: UPPER_CASE with underscores

### File Organization
- Directory structure standards
- Import organization rules
- File naming patterns

## Architecture Patterns

### Design Principles
- SOLID principles application
- Preferred design patterns
- Architecture decisions made

### Code Structure
- How to organize modules
- Dependency injection patterns
- Error handling standards

## Development Workflow

### Git Practices
- Branch naming conventions
- Commit message format
- PR/MR requirements

### Testing Standards
- Test file organization
- Coverage requirements
- Testing patterns to follow


### Local CI Testing
- Use `act` for local GitHub Actions testing when possible
- Fallback to direct cargo commands for reliable local verification
- Always run: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`
- Document any Docker/act configuration issues for team awareness
## Technology Constraints

### Libraries & Frameworks
- Approved libraries and versions
- Libraries to avoid and why
- Framework-specific patterns

### Performance Guidelines
- Performance targets
- Optimization priorities
- Resource usage limits

## API & Integration Rules

### API Design
- REST/GraphQL standards
- Authentication patterns
- Error response formats

### External Integrations
- Third-party service guidelines
- Data validation requirements
- Rate limiting strategies

## Security Guidelines

### Data Handling
- PII protection rules
- Data validation requirements
- Encryption standards

### Authentication & Authorization
- Auth implementation patterns
- Permission model rules
- Session management guidelines
