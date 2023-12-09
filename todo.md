# Todo

## Fix

- [ ] initial piece rotation breaks some hexes?
- [ ] focus over sprites sometimes lost?

## Houses

- [x] place houses
- [x] pieces can't be placed on houses
- [ ] randomly add neighbouring tiles to houses (iterate, but break on first failed rand check to prevent islands behind houses)

## Scoring

- [ ] check if houses are connected
- [ ] respawn grid on completion
- [ ] score completed grid
- [ ] allow resetting the board
- [ ] remove time when resetting board

## Revamp to routes

- [x] spawn from tilemap
- [x] spawn either 1 or 2 connected route hexes or 1 route + 1 empty
- [x] randomly rotate each hex
- [x] properly connect 2 route hexes (route on route action)
- [x] try triple hex pieces (triangles - 1 to 3 route hexes?)
- [x] handle hex & connection rotation
- [ ] randomly pick hex decorations for the empty hex

## Extra

- [ ] stagger piece spawns
- [ ] delay initial piece spawns (after board)
- [ ] refresh snapping on rotation
- [ ] fix pieces moving on rotation (tween them back)

## Story

???

## Basic loop

- [x] spawn test piece
- [x] move piece
- [x] snap piece to grid
- [x] don't snap when out of bounds
- [x] don't snap when placement is not possible (hexes occupied)
- [x] tween snapping
- [x] restore position on invalid drop
- [x] place hex
- [x] randomize pieces
- [x] check for lines
- [x] rotate pieces
- [x] rotate around the hovered hex
- [x] spawn 3 options
- [x] remove ingredient from hex variant
- [x] have to use 2 out of 3 pieces
