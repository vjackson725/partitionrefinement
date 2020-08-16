
//
// Implements Partition Refinement for Branching Bisimilarity, slowly
//

use std::collections::{HashSet, HashMap};

use std::hash::Hash;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Act {
  Tau,
  A(char)
}

type Part = usize;

type Node<'a> = &'a str;

type Edge<N> = (N, N, Act);

type Graph<N> = HashMap<N, HashSet<(N,Act)>>;


fn main() {
  use Act::Tau;

  let data : &[Edge<Node<'_>>] = &[
    ("00", "10", Tau),
    ("10", "20", Tau),
    ("20", "30", Act::A('b')),

    ("01", "11", Tau),
    ("11", "21", Tau),
    ("21", "31", Act::A('c')),

    ("02", "12", Tau),
    ("12", "22", Tau),

    ("13", "23", Tau),

    ("00", "01", Tau),
    ("01", "02", Tau),
    ("02", "03", Act::A('c')),

    ("10", "11", Tau),
    ("11", "12", Tau),
    ("12", "13", Act::A('c')),

    ("20", "21", Tau),
    ("21", "22", Tau),
    ("22", "23", Act::A('b')),

    ("30", "31", Tau),
  ];

  let graph = {
    let mut graph = HashMap::new();

    for (s,t,a) in data {
      // add t to s
      let s_edge = graph.entry(*s);
      let s_to = s_edge.or_insert_with(|| HashSet::new());
      s_to.insert((*t,*a));

      // ensure t has an entry
      graph.entry(*t).or_insert_with(|| HashSet::new());
    }
    graph
  };


  // partions are represented as a vector of true/false comparisons
  let mut max_part : Part = 0;
  let mut part_map : HashMap<Node<'_>, Part> = graph.keys().map(|n| (*n, max_part)).collect();

  // fixpoint computation begin
  loop {
    for (n, p) in &part_map {
      print!("{} | ", n);
      println!("{:?}", p);
    }
    println!();

    // compute splitters of all nodes
    let mut node_splitters_map = HashMap::<_,_>::new();
    for s in graph.keys() {
      let splitters = find_splitters(s, &graph, &part_map);
      node_splitters_map.insert(s, splitters);
    }

    for (n, splt) in &node_splitters_map {
      print!("{},{} | ", n, part_map.get(*n).unwrap());
      println!("{:?}", splt);
    }
    println!();


    // In each partition, union the splitters of the nodes.
    let partition_splitters_map : HashMap<Part, HashSet<(Act,Part)>> = {
      let mut m = HashMap::new();
      for (n, n_splitters) in &node_splitters_map {
        let n_part = part_map.get(*n).expect("n not in part_map");
        let part_splitters = m.entry(*n_part).or_insert_with(|| HashSet::new());

        part_splitters.extend(n_splitters);
      }
      m
    };

    // take all nodes, and compare the node splitters with the splitters of the
    // partition:
    //   * if they are the same, all is good;
    //   * if they are different, a split must occur within the partition.
    let mut active_splitter_conflicts : HashMap<Part, (&(Act, Part), Part)> = HashMap::new();

    for (n, p) in part_map.iter_mut() {
      let partition_splitters = partition_splitters_map.get(p).expect("n not in partition_splitters_map");
      let node_splitters = node_splitters_map.get(n).expect("n not in partition_splitters_map");

      if let Some((contested_splitter, new_part)) = active_splitter_conflicts.get(p) {
        if !node_splitters.contains(contested_splitter) {
          // nodes without the contested splitter (incl. this node) move to the new partition
          *p = *new_part;
        }
      } else {
        let contested_splitters : HashSet<_> = partition_splitters.difference(node_splitters).collect();

        if let Some(contested_splitter) = contested_splitters.iter().next() {
          // { !contested_splitters.is_empty() }

          // note that every considered node before this point
          // has had all splitters in partition_splitters.

          // nodes without this contested splitter (incl. this node) move to the new partition
          // nodes with this contested splitter will will stay the same (to remain consistent with previous nodes)

          let new_part = max_part+1;
          max_part += 1;

          active_splitter_conflicts.insert(*p, (contested_splitter, new_part));
          *p = new_part;
        }
      }
    }

    if active_splitter_conflicts.is_empty() {
      break;
    } else {
      for (p, (splt, pnew)) in &active_splitter_conflicts {
        println!("partition: {}, split on: {:?}, new part: {}", p, splt, pnew);
      }
      println!();
    }

    println!();
  }

  println!("done");
}

fn find_splitters<'a, N: Hash + Eq + Copy, P: Clone + Eq + Hash>(
  n : &'a N, graph : &'a Graph<N>,
  part : &'a HashMap<N, P>
) -> HashSet<(Act, P)>
{

  let mut active : HashSet<&N> = { let mut m = HashSet::new(); m.insert(n); m };
  let mut actions : HashSet<_> = HashSet::new();
  
  loop {
    let maybe_s = (&mut active).drain().next();
    if let Some(s) = maybe_s {

      let s_part = part.get(&s).expect("n ∉ part"); // safe as n ∈ part

      // for all reachable nodes:
      //   * if t is in a different partition, add s and the action to the output;
      //   * if t is in the same partition, and connected by τ, follow it (add to active).

      for (t,a) in graph.get(&s).expect("n ∉ graph").iter() // safe as n ∈ graph
      {
        let t_part = part.get(&t).unwrap(); // safe as all nodes have a partition

        match (*a, s_part == t_part) {
            (Act::Tau , false)
          | (Act::A(_), _)     => { actions.insert((*a, t_part.clone())); },
            (Act::Tau , true)  => { active.insert(t); },
        }
      }

    } else {
      break
    }
  }

  actions
}