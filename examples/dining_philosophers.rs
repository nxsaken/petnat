use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_ascii_terminal::{
    AutoCamera, Border, Terminal, TerminalBundle, TerminalPlugin, TileFormatter,
};

use petnat::{NetId, PetriNet, PetriNetPlugin, Place, Token, Trans, W};

enum DiningPhils {}

const LEFT: bool = true;
const RIGHT: bool = false;

enum ForkClean<const LR: bool> {}
enum ForkTaken<const LR: bool, const N: usize> {}
enum ForkDirty<const LR: bool, const N: usize> {}
enum Eating<const N: usize> {}
enum Thinking<const N: usize> {}

enum Take<const LR: bool, const N: usize> {}
enum Wash<const LR: bool, const N: usize> {}
enum Eat<const N: usize> {}
enum Finish<const N: usize> {}

impl NetId for DiningPhils {}

impl<const LR: bool> Place<DiningPhils> for ForkClean<LR> {}
impl<const LR: bool, const N: usize> Place<DiningPhils> for ForkTaken<LR, N> {}
impl<const LR: bool, const N: usize> Place<DiningPhils> for ForkDirty<LR, N> {}
impl<const N: usize> Place<DiningPhils> for Eating<N> {}
impl<const N: usize> Place<DiningPhils> for Thinking<N> {}

impl<const LR: bool, const N: usize> Trans<DiningPhils> for Take<LR, N> {}
impl<const LR: bool, const N: usize> Trans<DiningPhils> for Wash<LR, N> {}
impl<const N: usize> Trans<DiningPhils> for Eat<N> {}
impl<const N: usize> Trans<DiningPhils> for Finish<N> {}

fn add_philosopher<const N: usize>(net: PetriNet<DiningPhils>) -> PetriNet<DiningPhils> {
    net.add_place::<ForkTaken<LEFT, N>>()
        .add_place::<ForkTaken<RIGHT, N>>()
        .add_place::<ForkDirty<LEFT, N>>()
        .add_place::<ForkDirty<RIGHT, N>>()
        .add_place::<Eating<N>>()
        .add_place::<Thinking<N>>()
        .add_trans::<Take<LEFT, N>, (ForkClean<LEFT>, W<1>), (ForkTaken<LEFT, N>, W<1>)>()
        .add_trans::<Take<RIGHT, N>, (ForkClean<RIGHT>, W<1>), (ForkTaken<RIGHT, N>, W<1>)>()
        .add_trans::<Wash<LEFT, N>, (ForkDirty<LEFT, N>, W<1>), (ForkClean<LEFT>, W<1>)>()
        .add_trans::<Wash<RIGHT, N>, (ForkDirty<RIGHT, N>, W<1>), (ForkClean<RIGHT>, W<1>)>()
        .add_trans::<Eat<N>, (
            (Thinking<N>, W<1>),
            (ForkTaken<LEFT, N>, W<1>),
            (ForkTaken<RIGHT, N>, W<1>),
        ), (Eating<N>, W<1>)>()
        .add_trans::<Finish<N>, (Eating<N>, W<1>), (
            (Thinking<N>, W<1>),
            (ForkDirty<LEFT, N>, W<1>),
            (ForkDirty<RIGHT, N>, W<1>),
        )>()
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(640.0, 640.0),
                title: "Dining Philosophers".to_string(),
                resizable: LEFT,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TerminalPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(PetriNetPlugin::<DiningPhils> {
            build: |net| {
                net.add_place::<ForkClean<LEFT>>()
                    .add_place::<ForkClean<RIGHT>>()
                    .compose(add_philosopher::<0>)
                    .compose(add_philosopher::<1>)
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
                (show_manual, show_net),
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
                // In this setup, all transitions are fired manually via player input.
                take_fork::<LEFT, 0>.run_if(input_just_pressed(KeyCode::Q)),
                take_fork::<RIGHT, 0>.run_if(input_just_pressed(KeyCode::E)),
                wash_fork::<LEFT, 0>.run_if(input_just_pressed(KeyCode::A)),
                wash_fork::<RIGHT, 0>.run_if(input_just_pressed(KeyCode::D)),
                eat::<0>.run_if(input_just_pressed(KeyCode::W)),
                finish::<0>.run_if(input_just_pressed(KeyCode::S)),
                take_fork::<LEFT, 1>.run_if(input_just_pressed(KeyCode::U)),
                take_fork::<RIGHT, 1>.run_if(input_just_pressed(KeyCode::O)),
                wash_fork::<LEFT, 1>.run_if(input_just_pressed(KeyCode::J)),
                wash_fork::<RIGHT, 1>.run_if(input_just_pressed(KeyCode::L)),
                eat::<1>.run_if(input_just_pressed(KeyCode::I)),
                finish::<1>.run_if(input_just_pressed(KeyCode::K)),
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                show_clean_fork::<LEFT>,
                show_clean_fork::<RIGHT>,
                show_taken_fork::<LEFT, 0>,
                show_taken_fork::<LEFT, 1>,
                show_taken_fork::<RIGHT, 0>,
                show_taken_fork::<RIGHT, 1>,
                show_dirty_fork::<LEFT, 0>,
                show_dirty_fork::<LEFT, 1>,
                show_dirty_fork::<RIGHT, 0>,
                show_dirty_fork::<RIGHT, 1>,
                show_philosopher::<0>,
                show_philosopher::<1>,
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
    // A token can represent the current state of some game object, or some sort of resource.
    // In this example, the token represents the state of the dinner, capturing both the state
    // of the forks, and the state of the philosophers.
    let token = net.spawn_token();
    commands.spawn(token);
    info!("Spawning a token...");
}

fn init_forks(net: Res<PetriNet<DiningPhils>>, mut tokens: Query<&mut Token<DiningPhils>>) {
    let mut token = tokens.single_mut();
    net.mark::<ForkClean<LEFT>>(&mut token, 1);
    net.mark::<ForkClean<RIGHT>>(&mut token, 1);
}

fn init_philosopher<const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    net.mark::<Thinking<N>>(&mut token, 1);
}

const fn side<const LR: bool>() -> &'static str {
    match LR {
        LEFT => "left",
        RIGHT => "right",
    }
}

fn take_fork<const LR: bool, const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    if let Ok(()) = net.fire::<Take<LR, N>>(token.bypass_change_detection()) {
        token.set_changed();
        info!("Philosopher {N} took the {} fork.", side::<LR>());
    } else {
        warn!("Philosopher {N} cannot take the {} fork.", side::<LR>());
    }
}

fn wash_fork<const LR: bool, const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    if let Ok(()) = net.fire::<Wash<LR, N>>(token.bypass_change_detection()) {
        token.set_changed();
        info!("Philosopher {N} washed the {} fork.", side::<LR>());
    } else {
        warn!("Philosopher {N} cannot wash the {} fork.", side::<LR>());
    }
}

fn eat<const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    if let Ok(()) = net.fire::<Eat<N>>(token.bypass_change_detection()) {
        token.set_changed();
        info!("Philosopher {N} started eating.");
    } else if net.marks::<Eating<N>>(&token) > 0 {
        warn!("Philosopher {N} is already eating.");
    } else {
        warn!("Philosopher {N} cannot eat without both clean forks.");
    }
}

fn finish<const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    mut tokens: Query<&mut Token<DiningPhils>>,
) {
    let mut token = tokens.single_mut();
    if let Ok(()) = net.fire::<Finish<N>>(token.bypass_change_detection()) {
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
fn show_manual(mut terms: Query<&mut Terminal>) {
    let mut term = terms.single_mut();
    let ys = &mut (0..TERM_SIZE[1]).rev().take(19);
    let mut y = || ys.next().unwrap();
    term.put_string([0, y()], "               DINING PHILOSOPHERS                ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "                     Controls                     ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "     Philosopher 0              Philosopher 1     ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], " <Q> take left fork         <U> take left fork    ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], " <E> take right fork        <O> take right fork   ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], " <W> start eating           <I> start eating      ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], " <S> finish eating          <K> finish eating     ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], " <A> wash left fork         <J> wash left fork    ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], " <D> wash right fork        <L> wash right fork   ");
    term.put_string([0, y()], "                                                  ");
    term.put_string([0, y()], "--------------------------------------------------");
}

#[rustfmt::skip]
fn show_net(mut terms: Query<&mut Terminal>) {
    let mut term = terms.single_mut();
    let ys = &mut (0..TERM_SIZE[1]).rev().skip(20);
    let mut y = || ys.next().unwrap();
    term.put_string([0, y()], r"       WASH          FINISH           WASH       ");
    term.put_string([0, y()], r"                                                 ");
    term.put_string([0, y()], r"   +---[#]<---( )<-----[#]----->( )--->[#]---+   ");
    term.put_string([0, y()], r"   |         dirty     ^ \     dirty         |   ");
    term.put_string([0, y()], r"   |                  /   \                  |   ");
    term.put_string([0, y()], r"   |                 /     v                 |   ");
    term.put_string([0, y()], r"   |              ( )   0   ( )              |   ");
    term.put_string([0, y()], r"   |           eating^     /thinking         |   ");
    term.put_string([0, y()], r"   |  TAKE            \   /            TAKE  |   ");
    term.put_string([0, y()], r"   |                   \ v                   |   ");
    term.put_string([0, y()], r"   |   [#]--->( )----->[#]<-----( )<---[#]   |   ");
    term.put_string([0, y()], r"   |  ^      taken             taken      ^  |   ");
    term.put_string([0, y()], r"   | /                 EAT                 \ |   ");
    term.put_string([0, y()], r"   V/                                       \V   ");
    term.put_string([0, y()], r" L( )clean                             clean( )R ");
    term.put_string([0, y()], r"   ^\                                       /^   ");
    term.put_string([0, y()], r"   | \                 EAT                 / |   ");
    term.put_string([0, y()], r"   |  v                                   v  |   ");
    term.put_string([0, y()], r"   |   [#]--->( )----->[#]<-----( )<---[#]   |   ");
    term.put_string([0, y()], r"   |         taken     / ^     taken         |   ");
    term.put_string([0, y()], r"   |  TAKE            /   \            TAKE  |   ");
    term.put_string([0, y()], r"   |                 v     \                 |   ");
    term.put_string([0, y()], r"   |              ( )   1   ( )              |   ");
    term.put_string([0, y()], r"   |           eating\     ^thinking         |   ");
    term.put_string([0, y()], r"   |                  \   /                  |   ");
    term.put_string([0, y()], r"   |                   v /                   |   ");
    term.put_string([0, y()], r"   +---[#]<---( )<-----[#]----->( )--->[#]---+   ");
    term.put_string([0, y()], r"             dirty             dirty             ");
    term.put_string([0, y()], r"       WASH           FINISH          WASH       ");
}

const W: usize = TERM_SIZE[0];
const H: usize = TERM_SIZE[1] - 20;

const ROWS: [usize; 7] = [H - 3, H - 7, H - 11, H / 2, 11, 7, 3];
const COLS: [usize; 9] = [3, 8, 15, 19, W / 2 - 1, W - 21, W - 17, W - 10, W - 5];

fn show_place<P: Place<DiningPhils>>(
    net: &PetriNet<DiningPhils>,
    token: &Token<DiningPhils>,
    term: &mut Terminal,
    [x, y]: [usize; 2],
) {
    if net.marks::<P>(token) > 0 {
        term.put_char([x, y], '*'.fg(Color::YELLOW));
        term.put_char([x - 1, y], '('.fg(Color::YELLOW));
        term.put_char([x + 1, y], ')'.fg(Color::YELLOW));
    } else {
        term.clear_box([x, y], [1, 1]);
        term.put_char([x - 1, y], '('.fg(Color::WHITE));
        term.put_char([x + 1, y], ')'.fg(Color::WHITE));
    }
}

fn show_trans<T: Trans<DiningPhils>>(
    net: &PetriNet<DiningPhils>,
    token: &Token<DiningPhils>,
    term: &mut Terminal,
    [x, y]: [usize; 2],
) {
    if net.enabled::<T>(token) {
        term.put_char([x, y], '#'.fg(Color::GREEN));
        term.put_char([x - 1, y], '['.fg(Color::GREEN));
        term.put_char([x + 1, y], ']'.fg(Color::GREEN));
    } else {
        term.clear_box([x, y], [1, 1]);
        term.put_char([x - 1, y], '['.fg(Color::WHITE));
        term.put_char([x + 1, y], ']'.fg(Color::WHITE));
    }
}

fn show_clean_fork<const LR: bool>(
    net: Res<PetriNet<DiningPhils>>,
    tokens: Query<&Token<DiningPhils>, Changed<Token<DiningPhils>>>,
    mut terms: Query<&mut Terminal>,
) {
    if let Ok(token) = tokens.get_single() {
        let mut term = terms.single_mut();
        let x = if LR { COLS[0] } else { COLS[8] };
        show_place::<ForkClean<LR>>(&net, token, &mut term, [x, ROWS[3]]);
    }
}

fn show_taken_fork<const LR: bool, const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    tokens: Query<&Token<DiningPhils>, Changed<Token<DiningPhils>>>,
    mut terms: Query<&mut Terminal>,
) {
    let mut term = terms.single_mut();
    if let Ok(token) = tokens.get_single() {
        let x = if LR { COLS[2] } else { COLS[6] };
        let y = if N == 0 { ROWS[2] } else { ROWS[4] };
        show_place::<ForkTaken<LR, N>>(&net, token, &mut term, [x, y]);
        let x = if LR { COLS[1] } else { COLS[7] };
        show_trans::<Take<LR, N>>(&net, token, &mut term, [x, y]);
    }
}

fn show_dirty_fork<const LR: bool, const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    tokens: Query<&Token<DiningPhils>, Changed<Token<DiningPhils>>>,
    mut terms: Query<&mut Terminal>,
) {
    let mut term = terms.single_mut();
    if let Ok(token) = tokens.get_single() {
        let x = if LR { COLS[2] } else { COLS[6] };
        let y = if N == 0 { ROWS[0] } else { ROWS[6] };
        show_place::<ForkDirty<LR, N>>(&net, token, &mut term, [x, y]);
        let x = if LR { COLS[1] } else { COLS[7] };
        show_trans::<Wash<LR, N>>(&net, token, &mut term, [x, y]);
    }
}

fn show_philosopher<const N: usize>(
    net: Res<PetriNet<DiningPhils>>,
    tokens: Query<&Token<DiningPhils>, Changed<Token<DiningPhils>>>,
    mut terms: Query<&mut Terminal>,
) {
    let mut term = terms.single_mut();
    if let Ok(token) = tokens.get_single() {
        let y = if N == 0 { ROWS[1] } else { ROWS[5] };
        show_place::<Eating<N>>(&net, token, &mut term, [COLS[3], y]);
        show_place::<Thinking<N>>(&net, token, &mut term, [COLS[5], y]);
        let y = if N == 0 { ROWS[2] } else { ROWS[4] };
        show_trans::<Eat<N>>(&net, token, &mut term, [COLS[4], y]);
        let y = if N == 0 { ROWS[0] } else { ROWS[6] };
        show_trans::<Finish<N>>(&net, token, &mut term, [COLS[4], y]);
    }
}
