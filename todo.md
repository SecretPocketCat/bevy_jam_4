# Todo

## Visual overhaul

- [x] palette
- [ ] hexes
- [ ] BG
- [ ] font
- [ ] UI text
- [ ] btns

## Feedback

- [ ] add reset btn!
- [x] fix/refresh snapping on rotation
- [x] set higher z for dragged piece
- [ ] highligh selected & dragged piece
- [ ] remaining pieces UI

## Debug

- [x] reset map

## Revamp to routes

- [x] spawn from tilemap
- [x] spawn either 1 or 2 connected route hexes or 1 route + 1 empty
- [x] randomly rotate each hex
- [x] properly connect 2 route hexes (route on route action)
- [x] try triple hex pieces (triangles - 1 to 3 route hexes?)
- [x] handle hex & connection rotation
- [ ] randomly pick hex decorations for the empty hex

## Extra

- [ ] Pause menu
- [ ] Start with extra few single pieces
- [x] stagger piece spawns
- [x] delay initial piece spawns (after board)
- [ ] fix pieces moving on rotation (tween them back)

## Fix

- [ ] dragging sometimes breaks (the initial position then goes nuts)
- [x] resetting while dragging a piece causes a crash
- [x] initial piece rotation breaks some hexes?
- [x] focus over sprites sometimes lost
- [x] fix board animation/tweens

## Story

???

## Scaling

- scale based on level
  - [x] map size
  - [x] house count
  - [x] blocked hexes in the center

## Houses

- [x] place houses
- [x] pieces can't be placed on houses
- [x] randomly add neighbouring tiles to houses (iterate, but break on first failed rand check to prevent islands behind houses)

## Scoring

- [x] add score UI
- [x] add score on completion
- [x] add time UI
- [x] tick time
- [x] remove time when resetting board
- [x] final score state
- [x] reset stuff when the playing state is exited
- [x] show final score on the final score screen
- [x] add replay btn to final score

## Graph

- [x] check if houses are connected
- [x] respawn grid on completion
- [x] score completed grid
- [x] allow resetting the boards

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
