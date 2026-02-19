use spanning_tree::ui::App;

fn main() {
    App::run().unwrap();
    // let mut rng = rand::rng();
    // let mut g = Graph::new_test(
    //     vec![
    //         (TreeId::new_simple(0), TreeColor::of(0)),
    //         (TreeId::new_simple(3), TreeColor::of(0)),
    //         (TreeId::new_simple(2), TreeColor::of(0)),
    //         (TreeId::new_simple(3), TreeColor::of(1)),
    //     ],
    //     2,
    // );
    // let mut g = Graph::new_random(Configuration::new(10, 2), &mut rng);
    // println!("{:#?}", g.nodes());
    // for i in 1..=5 {
    //     println!("Round {i}!");
    //     g.execute_round(&mut rng);
    //     println!("{}", g.trees().iter().format(", "));
    // }
}
