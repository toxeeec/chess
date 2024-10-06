#!/bin/bash

# TODO: remove the script once drizzle adds support for creating strict tables and using `drizzle-kit push` with sqlite.

sqlite3 chess.db <<EOF
create table if not exists users (id text primary key) strict;
EOF
