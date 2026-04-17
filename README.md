# Project Catalog

A desktop application for browsing and running JavaFX AbaCord projects.

## Getting Started

### Option 1: Run from source

#### Prerequisites

Make sure you have installed:

- [Rust](https://www.rust-lang.org)
- [Git](https://git-scm.com)
- [Java](https://www.oracle.com/java)
- [Maven](https://maven.apache.org)

#### Steps

Clone the repo:

```bash
git clone git@github.com:AbaCord/project-catalog.git
cd project-catalog
```

Run the code (optionally add `--release` flag after `run`):

```bash
cargo run
```

### Option 2: Download executable

#### Prerequisites

Make sure you have installed:

- [Git](https://git-scm.com)
- [Java](https://www.oracle.com/java)
- [Maven](https://maven.apache.org)

#### Steps

1. Go to the [Latest Release](https://github.com/AbaCord/project-catalog/releases/latest)
2. Download the executable for your platform
3. Run it. Windows probably gives a warning about the executable, but you can trust it ;)

## How It Works

1. The app displays a list of JavaFX projects
2. When you select a project:
   - The repository is cloned to a platform specific cache folder using Git
   - If it's already cloned, no extra cloning will happen
   - The project is launched with Maven, resolving all dependencies
   - This might take some time initially, be patient (java is modern language)

## Requirements for Projects

To be compatible with the catalog, projects must:

- Be publicly accessible via Git

The following requirements are subject to change:

- Use Maven (`pom.xml`)
- Be runnable via `mvn javafx:run`
