## Overview

`cargo-projects` helps you manage your local Rust development environment by:
1. tracking your projects, 
2. monitoring disk usage, 
3. and providing easy cleanup capabilities. 
4. tracking/automating unstaged/unpushed code before you completely delete a project ^^

$ > cargo projects list

![Example output of cargo projects list](docs/screenshot.png)

Who is this for? Mainly me, but I think it can be useful for other developers as well who are working on multiple Rust projects locally and who want to keep their development environment organized, especially disk usage. 

My motivation for writing this project: My main workstaion only has 500GB storage and has rather weak compile hardware. I like to create various projects, compile them and leave them just there so I can come back later without compiling. That is especially useful if I work on bevy games and compare different games. At one point I hit my max storage and have to delete some stuff. Most of the time I just forgot which projects I didnt regularly use anymore, and which could be cleaned up by `cargo clean`. But I also forget how painfully long some projects take to compile, so this tool also helps me in managing if I should clean this one project up or not and how frequent I actually used it (or looked at it). 

I hope this tool helps people like me. 

## Features

- **List Projects**: View all your tracked Rust projects with detailed information
- **Project Tracking**: Automatically discover and track Rust projects
- **Disk Usage Monitoring**: Track project sizes including target directories
- **Cleanup Tools**: Remove unused projects and clean target directories
- **File System Watching**: Monitor directories for new Rust projects
- **Multiple Watchers**: Set up different watchers for different directories
- **Plugins-Configurability**:  (#TODO)
  - Add/Remove columns by adding your own plugins.
  - Customize the output of the command `cargo project lists` and others.