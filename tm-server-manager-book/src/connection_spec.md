# Leaderboard Modification Node

Fuse Connections: Would combine the data of all inputs.


# Input Node
The input node captures the input of the parent competition once all input connections resolve there.

# Reducers
```rust
fn 
```


# Connection Specification
Everything is enforced server side.

## Connections conditions 
Always have to be be made in the same parent comeptition.
Cannot result in a cycle (competitions contain a DAG (directed acyclic graph))
Cannot be made between normal and template nodes.


## Allowed Connection Combinations.
as origin = outgoing connections to another node.
as target = incoming connections from another node.
Everything = Wait, Action, Data.
### Schedule:
as origin: only wait and action connection.
as target: only wait (also has to be set to relative).
### Competition:
as origin: not allowed.
as target: Data and Wait.
### Input:
as origin: Everything.
as target: not allowed.
### Match:
as origin: Everything.
as target: Everything.
### Registration:
as origin: Everything.
as target: Action.
### Server:
as origin: not allowed.
as target: not allowed.