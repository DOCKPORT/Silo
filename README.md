![Silo](logo/silo_banner.jpg)

# Silo

Silo is a desktop backup application for Linux, built 100% in Rust with the Iced GUI framework. It lets you define a protected body of data — a "silo" — by selecting folders and exclude patterns, then mirror that silo to a destination of your choice with rsync. The interface carries a dark, sci-fi post-apocalyptic theme inspired by the TV show *Silo*.

> **Status: Early development.**
> This project is in the early stages of development.

## Planned features

- Populate a silo by selecting folders to protect.
- Persist silo settings (paths, excludes, destination, timestamps) to a local SQLite store under `~/.local`
- Mirror the silo to any local destination using rsync, with the destination kept as a 100% mirror of the source
- Minimal monitoring: overall size, item/file counts, and last-sync timestamp

## Tech stack

| Component | Technology |
|---|---|
| Language | Rust |
| GUI | Iced |
| Config store | SQLite (via rusqlite) |
| Sync engine | rsync |
| Platform | Linux |


