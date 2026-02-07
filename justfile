# Headspace CRM development tasks

set dotenv-load

# Default: list available recipes
default:
    @just --list

# Reset the database: drop, recreate, and start app (migrates + seeds automatically)
reset-db:
    @echo "Dropping database headspace..."
    dropdb --if-exists headspace
    @echo "Creating database headspace..."
    createdb headspace
    @echo "Database recreated. Run 'just run' to migrate and seed."
    @echo ""
    @echo "Credentials (created on first run):"
    @echo "  Super admin:  admin@headspace.local / admin123"
    @echo "  Tenant admin: admin@demo.headspace.local / admin123"

# Build frontend (Vue + Vite)
build-frontend:
    cd frontend && npm run build

# Run the app
run:
    cargo run

# Build frontend then run app
dev: build-frontend run

# Check both Rust and frontend compile
check:
    cargo build
    cd frontend && npm run build

# Reset database and immediately run the app
reset: reset-db run
