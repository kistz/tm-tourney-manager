# Connecting Servers
To make your servers accessible in the server manager you need to connect them.
This is done via a docker container:
- `tmservers-standalone`: Docker container with embedded trackmania binary. (WIP)
- `tmservers-bridge`: Companion Docker container to the trackmania image acting like every other server-controller out there.

Once the server has been connected it will appear as unverified in the dashboard.
Make sure to verify it and lend it to your desired competitions.