# pretty

pretty (PseudoREgalia Time Trial thingY) is a tool for generating time trials. It reads in a text file like this:

```
8000 15780 1100 bookshelf
12050 13080 1280 pillar1 < bookshelf
14400 10800 -100 pillar2 < bookshelf
10500 9360 900 pillar3 < bookshelf
10500 2940 1000 axe_room_exit < pillar1 pillar2 pillar3
4500 1540 2300 auditorium_balcony
-800 1540 1700 auditorium_back
```

... and stuffs that data into a cooked data table asset file which can be loaded into the game.

Once the data table is in the game, you also need to hook it up to a BP_CourseController (a custom blueprint I made which spawns the course waypoints and handles events). This tool does not handle that. One way to hook up the data table is to edit the imports of a cooked level asset that has a BP_CourseController in it. BP_CourseController should already be pointing to a data table, so you can edit the path of the import to point to the data table you generated with this tool. You can edit imports with [uedit](https://github.com/turncoda/uedit), among other things.
