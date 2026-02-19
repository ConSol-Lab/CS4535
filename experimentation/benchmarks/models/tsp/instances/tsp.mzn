include "globals.mzn";

% The number of nodes in the graph.
int: N;

% The distance matrix between every pair of nodes.
array [1..N, 1..N] of int: Dist;

% `Successor[i]` denotes the node which succeeds the node `i`.
array [1..N] of var 1..N: Successor;

% The objective we optimize for.
var int: Objective;

% Enforce a Hamiltonian cycle.
constraint circuit(Successor);

% Optimize for the shortest tour length.
constraint Objective = sum([Dist[node, Successor[node]] | node in 1..N]);

solve minimize Objective;
