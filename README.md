# pretty

pretty (PseudoREgalia Time Trial thingY) is a tool for generating time trials
in Pseudoregalia. It reads the data in a text file like this:

```
 8000   15780  1100 bookshelf
 12050  13080  1280 pillar1 < bookshelf
 14400  10800 -100  pillar2 < bookshelf
 10500  9360   900  pillar3 < bookshelf
 10500  2940   1000 axe_room_exit < pillar1 pillar2 pillar3
 4500   1540   2300 auditorium_balcony
-800    1540   1700 auditorium_back
```

... and stuffs that into a cooked data table asset file which can be loaded
into the game.

Once the data table is loaded into the game, you also need to hook it up to a
BP_CourseController (a custom blueprint I made which reads course data from a
data table, spawns the course waypoints and handles events). This tool does
not handle that. One way to hook up the data table is to edit the imports of a
cooked level asset that has a BP_CourseController in it. BP_CourseController
should already be pointing to a data table, so you can change the path of the
import it is currently using to the path of the data table you generated with
this tool. One tool you can use to edit imports is
[uedit](https://github.com/turncoda/uedit).

### Syntax

The input text file is how you specify the XYZ coordinates at which course
waypoints appear. If you just want a series of waypoints that unlock linearly
in sequence, then the XYZ coordinates are all you need. Something like this
will suffice:

```
100 200 300
400 500 600
...
```

If you want multiple waypoints to appear at the same time, you'll need to
specify "gates" &mdash; the waypoints which must be collected for a waypoint to
appear.

Let's look at an example:

```
100 100 100 A
200 200 200 B < A
```

This text file says:

- There is a waypoint labeled 'A' located at (100, 100, 100) gated by nothing
  (i.e. it spawns immediately when play begins)
- There is a waypoint labeled 'B' located at (200, 200, 200) gated by waypoint
  'A'

This is actually redundant because each waypoint is already gated on the
waypoint before it by default.

Let's look at a slightly more complex example:

```
100 100 100 A
200 200 200 B < A
300 300 300 C < A
400 400 400 D < B C
500 500 500 E
```

This text file says that:

- A spawns immediately when play begins
- Collecting A spawns B and C
- Collecting both B and C spawns D
- Collecting D spawns E (by default, because it doesn't explicitly specify
  its gates)

That about covers the major concepts. The full syntax description is as
follows:

```
X Y Z [ LABEL [ < GATE_1 GATE_2 ... GATE_N ] ]
```
