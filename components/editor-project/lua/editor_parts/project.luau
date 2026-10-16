-- Part: the Project panel, the project directory as a live file tree (the directory service).
local panels = require("panels")
return { name = "project", slot = "left", order = 1, size = "55%", render = panels.project }
