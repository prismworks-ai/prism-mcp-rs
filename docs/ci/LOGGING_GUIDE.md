# CI Container Logging Guide

## 📍 Log Locations and Access Methods

### 1. **Docker Container Logs (Primary Method)**

Docker containers write to stdout/stderr, which Docker captures. Access them using:

```bash
# View logs for a specific container
docker logs <container-name>

# Follow logs in real-time
docker logs -f <container-name>

# Show last N lines
docker logs --tail 50 <container-name>

# Show logs with timestamps
docker logs -t <container-name>

# Show logs since a specific time
docker logs --since 2h <container-name>  # last 2 hours
docker logs --since 2024-08-12T19:00:00 <container-name>  # since specific time
```

### 2. **Act-Specific Logs**

When running with `act`, logs are captured in multiple ways:

```bash
# View act container logs
docker logs act-CI-<job-name>-<hash>

# Example for Test Suite
docker logs act-CI-Test-Suite-1-<hash>

# List all act containers
docker ps --filter "name=act-CI"
```

### 3. **OrbStack Log Locations (macOS)**

OrbStack stores logs in:
- **OrbStack GUI logs**: `~/.orbstack/log/gui.log`
- **VM Manager logs**: `~/.orbstack/log/vmgr.log`
- **Container logs**: Accessed via `docker logs` command (not directly on filesystem)

### 4. **Persistent Log Storage (Custom Setup)**

To persist logs beyond container lifetime, mount volumes:

```yaml
# In docker-compose.yml
services:
  ci-runner:
    volumes:
      - ./logs/ci-runner:/var/log/ci
    environment:
      LOG_DIR: /var/log/ci
```

### 5. **Act Workflow Logs**

When running act with verbose mode:

```bash
# Run with verbose logging
act push -W .github/workflows/ci.yml --verbose 2>&1 | tee ci-run-$(date +%Y%m%d-%H%M%S).log

# Or use the helper script which includes verbose by default
./run-ci-with-network.sh 2>&1 | tee ci-run-$(date +%Y%m%d-%H%M%S).log
```

## 🔍 Useful Log Commands

### View All CI Container Logs
```bash
#!/bin/bash
# Save as view-ci-logs.sh
for container in $(docker ps -a --filter "name=act-CI" --format "{{.Names}}"); do
    echo "\n=== Logs for $container ==="
    docker logs --tail 50 $container
done
```

### Export All Logs
```bash
#!/bin/bash
# Save as export-ci-logs.sh
LOG_DIR="ci-logs-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$LOG_DIR"

for container in $(docker ps -a --filter "name=act-CI" --format "{{.Names}}"); do
    docker logs $container > "$LOG_DIR/$container.log" 2>&1
    echo "Exported logs for $container"
done

echo "All logs exported to $LOG_DIR/"
```

### Monitor Multiple Containers
```bash
# Using tmux or multiple terminals
docker logs -f act-CI-Test-Suite-1-<hash>
docker logs -f act-CI-Clippy-<hash>
docker logs -f act-CI-Documentation-<hash>
```

### Search Logs for Errors
```bash
# Search for errors across all CI containers
for container in $(docker ps -a --filter "name=act-CI" --format "{{.Names}}"); do
    echo "\n=== Checking $container for errors ==="
    docker logs $container 2>&1 | grep -i -E "error|fail|panic"
done
```

## 📊 Log Aggregation Options

### Option 1: Simple File-Based Logging
Add to `docker-compose.yml`:
```yaml
services:
  ci-runner:
    command: /bin/bash -c "exec > >(tee -a /logs/output.log) 2>&1 && tail -f /dev/null"
    volumes:
      - ./logs:/logs
```

### Option 2: JSON Logging
Configure Docker daemon for JSON logs:
```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  }
}
```

### Option 3: Centralized Logging (Advanced)
Use docker-compose with a logging service:
```yaml
services:
  log-collector:
    image: fluent/fluentd
    volumes:
      - ./fluentd/conf:/fluentd/etc
      - ./logs:/var/log/containers
    ports:
      - "24224:24224"

  ci-runner:
    logging:
      driver: fluentd
      options:
        fluentd-address: localhost:24224
        tag: ci.runner
```

## 🚨 Troubleshooting

### No Logs Available
```bash
# Check if container is running
docker ps -a | grep <container-name>

# Check container inspect for log configuration
docker inspect <container-name> | grep -A 5 LogConfig
```

### Logs Too Large
```bash
# Rotate logs
docker logs <container-name> > archived-$(date +%Y%m%d).log
docker restart <container-name>
```

### Real-time Debugging
```bash
# Attach to running container
docker exec -it <container-name> /bin/bash

# Watch processes
docker exec <container-name> ps aux

# Check resource usage
docker stats <container-name>
```

## 🎯 Best Practices

1. **Always use timestamps**: Include timestamps in your logs for debugging
2. **Set log rotation**: Prevent disk space issues with log rotation
3. **Use structured logging**: JSON logs are easier to parse and analyze
4. **Aggregate important logs**: Collect CI results in a central location
5. **Clean up old logs**: Implement log retention policies

## 📝 Quick Reference

| Command | Purpose |
|---------|--------|
| `docker logs <name>` | View container logs |
| `docker logs -f <name>` | Follow logs real-time |
| `docker logs --tail 100 <name>` | Last 100 lines |
| `docker logs --since 1h <name>` | Logs from last hour |
| `docker ps -a --filter "name=act-CI"` | List CI containers |
| `docker inspect <name> \| jq '.LogPath'` | Find log file path |
