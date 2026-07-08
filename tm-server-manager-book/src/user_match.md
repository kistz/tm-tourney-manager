# Match
The arguably most important node is the match.

## Configuration
The match has two configurations:
- pre match (optional): Active when the match is in preparation 
- match: Active when playing the match
These configurations are essentially just ordinary `MatchConfig`files determining the mode etc.

## Status
The match status shows how the match has progressed e.g. Preparation when the match became available, live when the match is ongoing or ended when the match has finished.

## Auto Provision
When the match becomes available (e.g. all input connections resolved) it automatically assigns a server from the active pool.

## Recoery
When the match gets interupted unexpectedly you can recover it.

## Match Records (WIP)
not yet implemented.
