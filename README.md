# Trackmania Tournament Manager (WIP)

## Structure

`tm-server-client`: Implements the GBX Remote 2 protocol to interact with a Trackmania server.
`tm-server-interface`: Implements the tm-server-client as a spacetimedb-client.
`tm-server-types`: Provides type abstraction over GBX Remote 2 for use by all other crates.
`tm-tourney-manager`: Implements a module for spacetimedb to host and configure Trackmania tournaments. 