# Discord Bot

A feature-rich Discord bot with various commands for fun, administration, and utility functions. Built with Rust using Serenity, Poise, and Tokio.

[![Docker](https://img.shields.io/badge/Docker-Ready-blue)](DOCKER.md)

## Key Features

- **Admin Commands**: Moderation commands like `warn`, `purge`, and channel settings
- **Fun Commands**: Image generation, random pictures, and the iconic `blame` command
- **Utility Commands**: Weather information, reminders, user info, and more
- **Availability System**: Record and manage user unavailability dates

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- PostgreSQL database
- Discord Bot Token

### Environment Setup

Copy the example environment file and adjust your settings:

```bash
cp .env.example .env
# Edit .env with your configuration
```

### Running the Bot

#### Locally

```bash
cargo run
```

#### Using Docker

For deploying with Docker (especially on headless Ubuntu servers), see [Docker Deployment Guide](DOCKER.md).

```bash
# Quick start with Docker
cp .env.docker .env
# Edit .env with your configuration
./deploy.sh
```

## Database Migration System

This project uses a file-based migration system to manage database schema changes. Migrations are automatically run at application startup.

### Migration Structure

Migrations are stored in the `migrations/` directory, organized by version numbers:

```
migrations/
  V1__initial_schema/
    001_create_schema_migrations.sql
    002_add_indexes.sql
    003_create_prepared_statements.sql
  V2__feature_x/
    001_new_table.sql
    002_populate_data.sql
```

Each migration folder follows the naming convention `V{version}__{description}` where:
- `{version}` is an integer representing the migration version
- `{description}` is a descriptive name using underscores instead of spaces

### Creating New Migrations

To create a new migration:

1. Create a new directory in `migrations/` following the naming convention
2. Add SQL files with a numeric prefix to ensure execution order
3. The migration will be applied on the next application startup

### Migration Status

You can check the current database schema version using:
- The `/dbschema` command for administrators in Discord
- The application logs at startup

## Development

### Project Structure

- `src/commands/` - Command implementations grouped by category
- `src/services/` - Core service implementations
- `src/config/` - Configuration handling
- `src/error.rs` - Error type definitions
- `src/main.rs` - Application entry point

### Adding New Commands

1. Create a new file in the appropriate command category directory
2. Implement the command using the Poise framework
3. Register the command in `src/commands/{category}/mod.rs`
4. Add the command to the command list in `src/commands.rs`

## License

MIT License
