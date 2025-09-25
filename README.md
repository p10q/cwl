# CloudWatch Logs CLI (cwl)

A powerful and intuitive command-line tool for interacting with AWS CloudWatch Logs. Stream logs in real-time, query historical data, and manage log groups with ease.

## Features

### MVP Features (Implemented)
✅ **Real-time log streaming** - Tail logs from CloudWatch in real-time
✅ **Historical queries** - Search through past logs with time ranges
✅ **Time range support** - Use flexible time formats (`--since 1h`, `--start`, `--end`)
✅ **String filtering** - Filter logs with simple patterns or CloudWatch filter syntax
✅ **List log groups** - Browse available log groups with pattern matching
✅ **Colored output** - Enhanced readability with syntax highlighting
✅ **Progress indicators** - Visual feedback during operations
✅ **AWS profile support** - Switch between multiple AWS accounts
✅ **Formatted table output** - Dynamic column detection with intelligent sorting

## Installation

### From Source
```bash
# Clone the repository
git clone https://github.com/yourusername/cwl.git
cd cwl

# Build the project
cargo build --release

# Install to your PATH
cargo install --path .
```

## Usage

### Basic Commands

#### List Available Log Groups
```bash
# List all log groups
cwl groups

# Filter log groups
cwl groups --filter "production"
```

#### Stream Logs in Real-Time
```bash
# Tail logs (last 5 minutes)
cwl tail /aws/lambda/my-function

# Follow logs continuously
cwl tail /aws/lambda/my-function --follow

# Filter while tailing
cwl tail /aws/lambda/my-function --filter "ERROR" --highlight
```

#### Query Historical Logs
```bash
# Query logs from the last hour
cwl query /aws/lambda/my-function --since 1h

# Query with specific time range
cwl query /aws/lambda/my-function --start "2024-01-01 10:00:00" --end "2024-01-01 12:00:00"

# Query with filter and limit
cwl query /aws/lambda/my-function --filter "user_id=12345" --limit 500

# Query with formatted table output (auto-detects JSON structure)
cwl query /aws/lambda/my-function --since 1h --formatted
```

#### Formatted Table Output

The `--formatted` flag automatically parses JSON log entries and displays them in a dynamic table:

```bash
# View logs as a formatted table with all JSON fields as columns
cwl query /aws/lambda/my-function --formatted --since 1h

# Combine with filters for focused analysis
cwl query /aws/ecs/my-app --formatted --filter "ERROR" --limit 100
```

**Features of formatted output:**
- Automatically detects all JSON fields across log entries
- Uses dot notation for nested fields (e.g., `payload.message`, `user.id`)
- Sorts columns by frequency (most common fields appear leftmost)
- Calculates optimal column widths
- Truncates long values with ellipsis
- Colored headers and separators for clarity

Example output:
```
timestamp               │ log_group     │ level │ message      │ payload.error
────────────────────────┼───────────────┼───────┼──────────────┼───────────────
2024-01-01 10:15:23.456 │ my-app        │ ERROR │ Failed auth  │ Invalid token
2024-01-01 10:15:24.789 │ my-app        │ WARN  │ Retry attempt│ Connection timeout
```

### Time Range Options

The tool supports multiple time formats:

- **Relative time**: `--since 1h`, `--since 30m`, `--since 2d`
- **ISO 8601**: `--start 2024-01-01T10:00:00Z`
- **Human-readable**: `--start "2024-01-01 10:00:00"`
- **Unix timestamps**: `--start 1704103200`

### AWS Configuration

#### Using AWS Profiles
```bash
# Use a specific AWS profile
cwl tail /aws/lambda/my-function --profile production

# Use a specific region
cwl tail /aws/lambda/my-function --region us-west-2
```

The tool respects standard AWS environment variables:
- `AWS_PROFILE`
- `AWS_REGION`
- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`

## Examples

### Monitor Application Errors
```bash
# Stream errors from multiple Lambda functions
cwl tail /aws/lambda/prod-api --filter "ERROR" --follow --highlight
```

### Investigate Issues
```bash
# Find all logs for a specific user in the last 24 hours
cwl query /aws/ecs/my-app --since 24h --filter "user_id=abc123"
```

### Quick Log Check
```bash
# Check recent logs (last 5 minutes by default)
cwl tail /aws/lambda/my-function
```

## Configuration

Create a configuration file at `~/.config/cwl/config.toml`:

```toml
[defaults]
region = "us-east-1"
output = "colored"
max_events = 1000

[profiles.production]
assume_role = "arn:aws:iam::123456789:role/ProdReader"
region = "us-west-2"

[aliases]
lambda-errors = "query /aws/lambda/* --filter ERROR --since 1h"
```

## Roadmap

### Phase 2: Enhanced Filtering
- [ ] Regex support
- [x] JSON field extraction (via --formatted flag)
- [ ] Multiple log group support
- [ ] Additional output formats (JSON, CSV)

### Phase 3: Advanced Features
- [ ] CloudWatch Insights integration
- [ ] Interactive TUI mode
- [ ] Local caching
- [ ] Configuration profiles

### Phase 4: Polish
- [ ] Shell completions (bash, zsh, fish)
- [ ] Man page generation
- [ ] Performance optimizations
- [ ] Plugin system

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with:
- [aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust) - AWS SDK for Rust
- [clap](https://github.com/clap-rs/clap) - Command-line argument parser
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [colored](https://github.com/mackwic/colored) - Terminal colors
- [indicatif](https://github.com/console-rs/indicatif) - Progress indicators