# Trackmania Tournament Manager (WIP)
The Goal of this project is to provide an unified backend for organizing all sort of trackmania tournaments.
Concretly it is implemented as a spacetimedb module allowing self-hosting or relying on a centrally hosted instance on spacetimes "maincloud".
This has a few advantages:
1. Unique identities for users and servers through trackmanias authentication.
2. Ability to generate a typed interface for multiple languages through spacetime.
3. Everything happening in matches gets recorded automatically and can be reconstructed.
4. Live updating weboscket based api for custom tournament frontends. 

## Project Structure
- `tm-server-types`: Provides type abstractions over GBX Remote 2 for use by all other crates or standalone.
- `tm-server-client`: Implements the GBX Remote 2 protocol to interact with a Trackmania server over xml-rpc.
- `tm-server-interface`: Implements a so called "sidecar" for spacetimedb taking the role "trackmania server as a db client". That means it subscribes to events from the tourney manager instance to control the associated tm server.
- `tm-tourney-manager`: Implements a spacetimedb module to host and configure Trackmania tournaments in a flexible and as unopinionated interface as possible. 
- `tm-tourney-manager-api`: Houses the generated types from spacetime in its own module to have a strong versioned dependency for the interface crate.