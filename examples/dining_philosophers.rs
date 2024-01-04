use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_ascii_terminal::{
    AutoCamera, Border, Terminal, TerminalBundle, TerminalPlugin, TileFormatter,
};

use petnat::{NetId, PetriNet, PetriNetBuilder, PetriNetPlugin, Place, Token, Trans, W};

enum DiningPhils {}

enum ForkFree<const LR: bool> {}
enum ForkTaken<const LR: bool, const N: usize> {}
enum ForkUsed<const LR: bool, const N: usize> {}
enum Eating<const N: usize> {}
enum Thinking<const N: usize> {}

enum Take<const LR: bool, const N: usize> {}
enum Return<const LR: bool, const N: usize> {}
enum Eat<const N: usize> {}
enum Finish<const N: usize> {}

impl NetId for DiningPhils {}

impl<const LR: bool> Place<DiningPhils> for ForkFree<LR> {}
impl<const LR: bool, const N: usize> Place<DiningPhils> for ForkTaken<LR, N> {}
impl<const LR: bool, const N: usize> Place<DiningPhils> for ForkUsed<LR, N> {}
impl<const N: usize> Place<DiningPhils> for Eating<N> {}
impl<const N: usize> Place<DiningPhils> for Thinking<N> {}

impl<const LR: bool, const N: usize> Trans<DiningPhils> for Take<LR, N> {}
impl<const LR: bool, const N: usize> Trans<DiningPhils> for Return<LR, N> {}
impl<const N: usize> Trans<DiningPhils> for Eat<N> {}
impl<const N: usize> Trans<DiningPhils> for Finish<N> {}

fn add_philosopher<const N: usize>(
    builder: PetriNetBuilder<DiningPhils>,
) -> PetriNetBuilder<DiningPhils> {
    builder
        .add_place::<ForkTaken<true, N>>()
        .add_place::<ForkTaken<false, N>>()
        .add_place::<ForkUsed<true, N>>()
        .add_place::<ForkUsed<false, N>>()
        .add_place::<Eating<N>>()
        .add_place::<Thinking<N>>()
        .add_trans::<Take<true, N>, (ForkFree<true>, W<1>), (ForkTaken<true, N>, W<1>)>()
        .add_trans::<Take<false, N>, (ForkFree<false>, W<1>), (ForkTaken<false, N>, W<1>)>()
        .add_trans::<Return<true, N>, (ForkUsed<true, N>, W<1>), (ForkFree<true>, W<1>)>()
        .add_trans::<Return<false, N>, (ForkUsed<false, N>, W<1>), (ForkFree<false>, W<1>)>()
        .add_trans::<Eat<N>, (
            (Thinking<N>, W<1>),
            (ForkTaken<true, N>, W<1>),
            (ForkTaken<false, N>, W<1>),
        ), (Eating<N>, W<1>)>()
        .add_trans::<Finish<N>, (Eating<N>, W<1>), (
            (Thinking<N>, W<1>),
            (ForkUsed<true, N>, W<1>),
            (ForkUsed<false, N>, W<1>),
        )>()
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(640.0, 640.0),
                title: "Dining Philosophers".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TerminalPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(PetriNetPlugin::<DiningPhils> {
            build: |builder| {
                let builder = builder
                    .add_place::<ForkFree<true>>()
                    .add_place::<ForkFree<false>>();
                let builder = add_philosopher::<0>(builder);
                add_philosopher::<1>(builder)
            },
        })
        .add_systems(Startup, spawn_terminal)
        .add_systems(
            PostStartup,
            (
                (
                    spawn_token,
                    apply_deferred,
                    (init_forks, init_philosopher::<0>, init_philosopher::<1>),
                )
                    .chain(),
                (draw_manual, draw_net),
            ),
        )
        .add_systems(
            PreUpdate,
            (
                despawn_token,
                spawn_token,
                apply_deferred,
                (init_forks, init_philosopher::<0>, init_philosopher::<1>),
            )
                .chain()
                .run_if(input_just_pressed(KeyCode::G)),
        )
        .add_systems(
            Update,
            (
                take_fork::<true, 0>.run_if(input_just_pressed(KeyCode::Q)),
                take_fork::<false, 0>.run_if(input_just_pressed(KeyCode::E)),
                return_fork::<true, 0>.run_if(input_just_pressed(KeyCode::A)),
                return_fork::<false, 0>.run_if(input_just_pressed(KeyCode::D)),
                eat::<0>.run_if(input_just_pressed(KeyCode::W)),
                finish::<0>.run_if(input_just_pressed(KeyCode::S)),
                take_fork::<true, 1>.run_if(input_just_pressed(KeyCode::U)),
                take_fork::<false, 1>.run_if(input_just_pressed(KeyCode::O)),
                return_fork::<true, 1>.run_if(input_just_pressed(KeyCode::J)),
                return_fork::<false, 1>.run_if(input_just_pressed(KeyCode::L)),
                eat::<1>.run_if(input_just_pressed(KeyCode::I)),
                finish::<1>.run_if(input_just_pressed(KeyCode::K)),
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                |tokens: Query<(), Changed<Token<DiningPhils>>>| {
                    if tokens.get_single().is_ok() {
                        info!("=== STATE ===");
                    }
                },
                show_free_fork::<true>,
                show_free_fork::<false>,
                show_taken_fork::<true, 0>,
                show_taken_fork::<true, 1>,
                show_taken_fork::<false, 0>,
                show_taken_fork::<false, 1>,
                show_philosopher::<0>,
                show_philosopher::<1>,
                |tokens: Query<(), Changed<Token<DiningPhils>>>| {
                    if tokens.get_single().is_ok() {
                        info!("=============");
                    }
                },
            )
                .chain(),
        )
        .run();
}

const TERM_SIZE: [usize; 2] = [50, 50];

fn spawn_terminal(mut commands: Commands) {
    let term = Terminal::new(TERM_SIZE).with_border(Border::single_line());
    commands.spawn((TerminalBundle::from(term), AutoCamera));
}

fn spawn_token(mut commands: Commands, net: Res<PetriNet<DiningPhils>>) {
    // A token can represent the current state of some game object,
    // or some sort of resource
    let token = net.spawn_token();
    commands.spawn(token);
    info!("Spawning a token...");
}

fn init_forks(net: Res<PetriNet<DiningPhils>>, mut tokens: Query<&mut Token<DiningPhils>>) {
    let mut token = tokens.single_mut();
    net.mark::<ForkFree<true>>(&mut token, 1);
    net.mark::<ForkFree<false>>(&mut token, 1);
}

fn init_philosopher<const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    net.mark::<Thinking<N>>(&mut token, 1);
}

fn take_fork<const LR: bool, const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    if let Some(()) = net.fire::<Take<LR, N>>(token.bypass_change_detection()) {
        token.set_changed();
        info!("Philosopher {N} took the {} fork.", side::<LR>());
    } else {
        warn!("Philosopher {N} cannot take the {} fork.", side::<LR>());
    }
}

fn return_fork<const LR: bool, const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    if let Some(()) = net.fire::<Return<LR, N>>(token.bypass_change_detection()) {
        token.set_changed();
        info!("Philosopher {N} returned the {} fork.", side::<LR>());
    } else {
        warn!("Philosopher {N} cannot return the {} fork.", side::<LR>());
    }
}

fn eat<const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    if let Some(()) = net.fire::<Eat<N>>(token.bypass_change_detection()) {
        token.set_changed();
        info!("Philosopher {N} started eating.");
    } else if token.marks::<Eating<N>>() > 0 {
        warn!("Philosopher {N} is already eating.");
    } else {
        warn!("Philosopher {N} cannot eat without both forks.");
    }
}

fn finish<const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    if let Some(()) = net.fire::<Finish<N>>(token.bypass_change_detection()) {
        token.set_changed();
        info!("Philosopher {N} finished eating.");
    } else {
        warn!("Philosopher {N} cannot finish eating.");
    }
}

fn despawn_token(mut commands: Commands, tokens: Query<Entity, With<Token<DiningPhils>>>) {
    let token = tokens.single();
    commands.entity(token).despawn();
    info!("Despawning the token...");
}

#[rustfmt::skip]
fn draw_manual(mut terms: Query<&mut Terminal>) {
    let mut term = terms.single_mut();
    let ys = &mut (0..TERM_SIZE[1]).rev().take(19);
    let mut y = || ys.next().unwrap();
    term.put_string([0, y()], "              DINING PHILOSOPHERS                 ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "                    Controls                      ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "    Philosopher 0              Philosopher 1      ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "<Q> take left fork         <U> take left fork     ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "<E> take right fork        <O> take right fork    ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "<W> start eating           <I> start eating       ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "<S> finish eating          <K> finish eating      ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "<A> return left fork       <J> return left fork   ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "<D> return right fork      <L> return right fork  ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "--------------------------------------------------");
}

#[rustfmt::skip]
fn draw_net(mut terms: Query<&mut Terminal>) {
    let mut term = terms.single_mut();
    let ys = &mut (0..TERM_SIZE[1]).rev().skip(20);
    let mut y = || ys.next().unwrap();
    term.put_string([0, y()], r"       RET           FINISH            RET       ");
    term.put_string([0, y()], r"                                                 ");
    term.put_string([0, y()], r"   ----[ ]<---( )<-----[ ]----->( )--->[ ]----   ");
    term.put_string([0, y()], r"   |         used      ^ \      used         |   ");
    term.put_string([0, y()], r"   |                  /   \                  |   ");
    term.put_string([0, y()], r"   |                 /     v                 |   ");
    term.put_string([0, y()], r"   |              ( )   0   ( )              |   ");
    term.put_string([0, y()], r"   |          eating ^     / thinking        |   ");
    term.put_string([0, y()], r"   |  TAKE            \   /            TAKE  |   ");
    term.put_string([0, y()], r"   |                   \ v                   |   ");
    term.put_string([0, y()], r"   |   [ ]--->( )----->[ ]<-----( )<---[ ]   |   ");
    term.put_string([0, y()], r"   |  ^     taken               taken     ^  |   ");
    term.put_string([0, y()], r"   | /                 EAT                 \ |   ");
    term.put_string([0, y()], r"   V/                                       \V   ");
    term.put_string([0, y()], r"L ( )                                       ( ) R");
    term.put_string([0, y()], r"   ^\                                       /^   ");
    term.put_string([0, y()], r"   | \                 EAT                 / |   ");
    term.put_string([0, y()], r"   |  v                                   v  |   ");
    term.put_string([0, y()], r"   |   [ ]--->( )----->[ ]<-----( )<----[ ]   |   ");
    term.put_string([0, y()], r"   |        taken      / ^      taken        |   ");
    term.put_string([0, y()], r"   |  TAKE            /   \            TAKE  |   ");
    term.put_string([0, y()], r"   |                 v     \                 |   ");
    term.put_string([0, y()], r"   |              ( )   1   ( )              |   ");
    term.put_string([0, y()], r"   |          eating \     ^ thinking        |   ");
    term.put_string([0, y()], r"   |                  \   /                  |   ");
    term.put_string([0, y()], r"   |                   v /                   |   ");
    term.put_string([0, y()], r"   ----[ ]<---( )<-----[ ]----->( )--->[ ]----   ");
    term.put_string([0, y()], r"                                                 ");
    term.put_string([0, y()], r"       RET            FINISH           RET       ");
}

fn show_free_fork<const LR: bool>(
    tokens: Query<&Token<DiningPhils>, Changed<Token<DiningPhils>>>,
    mut terms: Query<&mut Terminal>,
) {
    let mut term = terms.single_mut();
    let coord = if LR {
        [3, TERM_SIZE[1] - 20 - 15]
    } else {
        [TERM_SIZE[0] - 5, TERM_SIZE[1] - 20 - 15]
    };
    if let Ok(token) = tokens.get_single() {
        if token.marks::<ForkFree<LR>>() > 0 {
            info!("{} fork: on the table", side::<LR>());
            term.put_char(coord, '*'.fg(Color::WHITE));
        } else {
            term.clear_box(coord, [1, 1]);
        }
    }
}

fn show_taken_fork<const LR: bool, const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    tokens: Query<&Token<DiningPhils>, Changed<Token<DiningPhils>>>,
    mut terms: Query<&mut Terminal>,
) {
    let mut term = terms.single_mut();
    let x = if LR { 15 } else { TERM_SIZE[0] - 17 };
    let y_taken = match N {
        0 => TERM_SIZE[1] - 20 - 11,
        1 => TERM_SIZE[1] - 20 - 19,
        _ => unreachable!(),
    };
    let y_used = match N {
        0 => TERM_SIZE[1] - 20 - 3,
        1 => TERM_SIZE[1] - 20 - 27,
        _ => unreachable!(),
    };
    if let Ok(token) = tokens.get_single() {
        if token.marks::<ForkTaken<LR, N>>() > 0 {
            info!("{} fork: taken by Philosopher {N}", side::<LR>());
            term.put_char([x, y_taken], '*'.fg(Color::WHITE));
        } else {
            term.clear_box([x, y_taken], [1, 1]);
        }
        if token.marks::<ForkUsed<LR, N>>() > 0 {
            info!("{} fork: used by Philosopher {N}", side::<LR>());
            term.put_char([x, y_used], '*'.fg(Color::WHITE));
        } else {
            term.clear_box([x, y_used], [1, 1]);
        }
        let x = if LR { 8 } else { TERM_SIZE[0] - 10 };
        if net.enabled::<Take<LR, N>>(token) {
            term.put_char([x, y_taken], '='.fg(Color::GREEN).bg(Color::GREEN));
        } else {
            term.clear_box([x, y_taken], [1, 1]);
        }
        if net.enabled::<Return<LR, N>>(token) {
            term.put_char([x, y_used], '='.fg(Color::GREEN).bg(Color::GREEN));
        } else {
            term.clear_box([x, y_used], [1, 1]);
        }
    }
}

fn show_philosopher<const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    tokens: Query<&Token<DiningPhils>, Changed<Token<DiningPhils>>>,
    mut terms: Query<&mut Terminal>,
) {
    let mut term = terms.single_mut();
    let x_eating = 19;
    let x_thinking = 29;
    let y = match N {
        0 => TERM_SIZE[1] - 20 - 7,
        1 => TERM_SIZE[1] - 20 - 23,
        _ => unreachable!(),
    };
    if let Ok(token) = tokens.get_single() {
        if token.marks::<Eating<N>>() > 0 {
            info!("Philosopher {N}: eating");
            term.put_char([x_eating, y], '*'.fg(Color::WHITE));
        } else {
            term.clear_box([x_eating, y], [1, 1]);
        }
        if token.marks::<Thinking<N>>() > 0 {
            info!("Philosopher {N}: thinking");
            term.put_char([x_thinking, y], '*'.fg(Color::WHITE));
        } else {
            term.clear_box([x_thinking, y], [1, 1]);
        }
        let x = 24;
        let y_eat = match N {
            0 => TERM_SIZE[1] - 20 - 11,
            1 => TERM_SIZE[1] - 20 - 19,
            _ => unreachable!(),
        };
        if net.enabled::<Eat<N>>(token) {
            term.put_char([x, y_eat], '='.fg(Color::GREEN).bg(Color::GREEN));
        } else {
            term.clear_box([x, y_eat], [1, 1]);
        }
        let y_think = match N {
            0 => TERM_SIZE[1] - 20 - 3,
            1 => TERM_SIZE[1] - 20 - 27,
            _ => unreachable!(),
        };
        if net.enabled::<Finish<N>>(token) {
            term.put_char([x, y_think], '='.fg(Color::GREEN).bg(Color::GREEN));
        } else {
            term.clear_box([x, y_think], [1, 1]);
        }
    }
}

const fn side<const LR: bool>() -> &'static str {
    match LR {
        true => "left",
        false => "right",
    }
}
