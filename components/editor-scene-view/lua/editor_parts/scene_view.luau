-- Part: the Scene view the host composites the running scene into.
local panels = require("panels")
return { name = "scene-view", key = "scene", slot = "center", order = 1, render = panels.scene_view }
