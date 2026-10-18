use bevy::prelude::*;
use neopets_data::{Disc, iso_path};

#[derive(Resource)]
struct DiscStatus(String);

fn main() {
    let mut args = std::env::args().skip(1);
    let mut explicit = None;
    let mut smoke_test = false;
    while let Some(arg) = args.next() {
        if arg == "--iso" {
            explicit = Some(
                args.next()
                    .unwrap_or_else(|| {
                        eprintln!("--iso requires a path");
                        std::process::exit(2)
                    })
                    .into(),
            );
        } else if arg == "--smoke-test" {
            smoke_test = true;
        } else {
            eprintln!("Unknown argument: {arg}");
            std::process::exit(2);
        }
    }
    let root = std::env::current_dir().expect("Current directory");
    let result = iso_path(&root, explicit).and_then(|path| {
        let mut disc = Disc::open(&path)?;
        let name = disc.verify_neopets()?;
        Ok(format!(
            "{name}\n{} disc files indexed. Opened read-only.",
            disc.files().len()
        ))
    });
    let status = match result {
        Ok(s) => s,
        Err(e) => format!("ISO not ready\n{e}"),
    };
    let mut app = App::new();
    app.insert_resource(DiscStatus(status))
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.06)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Neopets • Rust rewrite workspace".into(),
                resolution: (1000, 600).into(),
                visible: !smoke_test,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup);
    if smoke_test {
        app.insert_resource(bevy::winit::WinitSettings::continuous());
        app.add_systems(Update, smoke_exit);
    }
    app.run();
}

fn smoke_exit(frame: Res<bevy::diagnostic::FrameCount>, mut exit: MessageWriter<AppExit>) {
    if frame.0 >= 5 {
        info!("Startup smoke check passed; exiting after five frames.");
        exit.write(AppExit::Success);
    }
}

fn setup(mut commands: Commands, status: Res<DiscStatus>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text::new(format!("NEOPETS\nThe Darkest Faerie — rewrite workspace\n\n{}\n\nFoundation ready\nArea rendering and gameplay are not implemented yet.\n\nOriginal content comes from your ISO. No BIOS is used.", status.0)),
        TextFont { font_size: bevy::text::FontSize::Px(24.0), ..default() },
        TextColor(Color::srgb(0.87, 0.91, 0.97)),
        Node { margin: UiRect::all(px(40)), max_width: percent(90), ..default() },
    ));
}
