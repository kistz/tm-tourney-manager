# Node Based Architecture
At the heart of the application lies the decision to expose functionality through a node-based system.
The means that things (like matches or registrations) are broken down into blocks which are interconnectable and can be wired together with connections forming a node-graph which defines how your tournament flows through those nodes.
These node-graphs are encapsulated in Competitions, which is simultaniously the first node type.