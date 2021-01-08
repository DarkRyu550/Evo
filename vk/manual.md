# Evo: Simulation Manual
This manual describes in detail both the model and implementation of the 
simulation found in Evo.

## Concept
Conceptually, the simulation in this program models two insect colonies. One
colony of herbivores, that feed on plants provided by the environment, and
one colony of predators, who exclusively feed on the herbivores. For brevity, 
the herbivore group will be referred to as "Group H" from now on, likewise,
the predator group will be referred to as "Group P".

Both individual groups inhabit a plane, on which they have full freedom of 
movement in both the horizontal and vertical axises. Individuals are considered
to be points, that occupy no area, thus, any number of individuals may occupy 
the same position in the plane and there are no collisions between them. Each 
individual is given a field of view, which allows them to gather information
from the region of the plane that is immediately around them.

At each round of the simulation, every individual chooses, based on the 
information they gather from their surroundings, a direction and amplitude of 
movement, and an amount of pheromones to leave behind. These pheromones are made 
out of a combination of three chemicals, the amount of which is set for every
individual throughout their lives.

The compositing chemicals of the pheromone are the Red, Green and Blue 
chemicals, which upon being deposited onto the plane, will immediately begin
decomposing. The rate of which may be different for each chemical, depending
on the parameters set for the simulation.

## Individual Behavior
The behavior of every individual alive is calculated at the same time, every
iteration. For the sake of fairness, it is also guaranteed that all individuals
can only react to the sum of all the actions that were taken up to the last 
iteration of the simulation, conceptually, this can be thought of as a way to
model reaction times.

The field of view for an individual is a circle centered at its current 
position and with a radius given by the `ViewRadius` simulation parameter.

From within the fields of view of every individual, the simulation program will
condense the information it can gather into the following parameters, which 
will then be fed to the individuals to make their decisions:

| Parameter | Type | Description |
| :-------- | :--- | :---------- |
| Velocity         | `vec2`  | The vector of the last move. |
| Red Gradient     | `vec2`  | The gradient direction of the red chemical on the plane.         |
| Red Intensity    | `float` | The intensity of the change the red gradient represents .        |
| Green Gradient   | `vec2`  | The gradient direction of the green chemical on the plane.       |
| Green Intensity  | `float` | The intensity of the change the green gradient represents.       |
| Blue Gradient    | `vec2`  | The gradient direction of the blue chemical on the plane.        |
| Blue Intensity   | `float` | The intensity of the change the blue gradient represents .       |
| Alpha Gradient   | `vec2`  | The gradient direction of the [alpha channel][1] on the plane.   |
| Alpha Intensity  | `float` | The intensity of the change the [alpha][1] gradient represents . |
[1]: #the-alpha-component

### Internal Values
The following extra parameters are used to keep track of the state of a given individual:

| Parameter | Type | Description |
| :-------- | :--- | :---------- |
| Position  | `vec2`  | The position of the individual in world space.  |
| Velocity  | `vec2`  | Used to feed the input component.               |
| Energy    | `float` | The amount of energy this individual may spend. |


### The alpha component
The alpha component, on the plane, is used for the amount of grass available for the herbivores
to eat. This resource is limited and regenerates over time. Upon eating from a patch of grass, a
herbivore will have its energy parameter replenish to the maximum value.

### Walking and Starvation
Every individual needs energy to live and, thus, spends some of its energy reserves
on its metabolic processes. The amount of energy spent per unit of time and iteration
will vary with the immediate velocity of the individual at any given point in time.

Upon reaching a negative value of energy, an individual will starve to death.

### Reproduction
When an individual reaches a given energetic goal, it will choose to spend some of
that energy in making offspring. Reproduction works by combining the genetic 
parameters of both the individual and its mate, chosen by picking off the individual
that has the most energy, apart from the individual itself.

### Predation
Predation occurs when a predator gets close enough to a herbivore. The mechanism is
the same as with the herbivore feeding off the ground, just with a different source
of nutrition.
