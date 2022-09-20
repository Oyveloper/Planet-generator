# Planet generator (WIP)
The aim of this project is to be able to proceduraly generate planets based on user input. 
The planet generation will layer noise in order to create levels of detail. 
Hopefully it will be available online soon with the power of WebAssembly.

## Creating the sphere
To create a sphere base for the topology I am using the approach of inflating a cube. This avoids some problems you get with other ways of creating the mesh, for example uneven UV's and inconsistent vertices. 
The method is heavily inspired by [ Sebastian Lauge's series on generating planets ](https://www.youtube.com/c/SebastianLague).

## Game engine
I am using the Bevy game engine in Rust. It is still fairly new, but great for exploration projects like this. 
