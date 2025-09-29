spacetime delete tourney-manager
spacetime publish -p tm-tourney-manager tourney-manager
spacetime generate --lang rust --out-dir tm-tourney-manager-api/src/generated --project-path tm-tourney-manager
spacetime call tourney-manager create_tournament TestTourney
spacetime call tourney-manager add_event "Discovery#1" 1 null
spacetime call tourney-manager add_stage "TimeAttack" 1 null 
spacetime call tourney-manager provision_match 1 null false
spacetime call tourney-manager assign_server 1 "test"
spacetime sql tourney-manager "SELECT * FROM tournament"
spacetime sql tourney-manager "SELECT * FROM tournament_event"
spacetime sql tourney-manager "SELECT * FROM event_stage"
spacetime sql tourney-manager "SELECT * FROM stage_match"