# uswindpowerstats
REST API and web interfaces, in various different languages, for US wind turbine installations

## Database

MS-SQL on Linux under Docker using the official image.

## data_sources folder

Culled from https://eerscmap.usgs.gov/uswtdb/data/

Contains a CSV file with one row for every (well, most) turbines in the USA.

Also contains a CSV file with a list of US states.

## dataloader

Rust program to load the US states CSV and turbine CSV.
Uses https://github.com/prisma/tiberius to talk to SQL server.
Also uses Tokio, because Tiberius is async.

