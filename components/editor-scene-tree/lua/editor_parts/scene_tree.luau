-- Part: the Scene Tree panel, the selected scene's node tree.
local panels = require("panels")
return { name = "scene-tree", key = "tree", slot = "left", order = 2, render = panels.tree }
