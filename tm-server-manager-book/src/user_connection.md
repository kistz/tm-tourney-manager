# Connection
You can define a connection between two nodes as long as the underlying graph stays acyclic.
The connection also has a direction.
Whenever all incoming connections to a node has finished (e.g. a matches) the node can be started.
These connections allow you to specify players which will get passed in the next node.

## Configuring Connections
There is a `Wait`and a `Data` connection. If data is chosen you can define which players will be passed e.g. first n or similar.
