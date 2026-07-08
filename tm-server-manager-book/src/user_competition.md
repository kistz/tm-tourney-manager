# Competition
The competition is the primary unit of functionality and always holds one node-graph.

## Comeptition Nesting
Because a competition is a node type you can them inside each other forming a tree.

## Access Control
Each competition can define custom Roles and Permissions for users providing various granular capabilities sucha as creating new nodes or spectating matches etc.
These Permissions will also be inherited downwards the competition tree.

## Server Pool
Each competition has access to its own server pool which enables to assign and use your connected Trackmania Servers to specific competitions.
This server pool is inherited upwards the competition tree.

## Shared Configurations
Configurations for matches can be associated with a competition to allow the sharing between multiple nodes at once. Most useful for templates discussed later.

## Competition Records (WIP)
not yet implemented.
