
#### Foundations

 - Allow multiple scenes worth of graphics resources to be instantiated
 - Make loading new scenes asynchronous (another thread) (clear the scene queue I guess)
 - Check here if a load operation is underway and only proceed updating the current scene
   if it hasn't requested another scene be pushed (but some updating should be allowed?
   Say o allow a loading screen?)

#### Issues

- Error logged on exit; not all allocations freed before destruction of a memory block
- Resizing the window fails around half the time
