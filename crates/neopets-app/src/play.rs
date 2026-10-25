//! Play mode (M7 groundwork): a hero outfit placed at a world pack's spawn (SPEC-0006), moving by
//! the locomotion of neopets-sim (SPEC-0033 tiers and turn, SPEC-0040 root-step travel) over the
//! ground probe (SPEC-0018), animated by its action's clips (SPEC-0005).
use crate::character::HeroAnim;
use neopets_data::archive::ArchiveIndex;
use neopets_data::ground::{self, CellOrder, Ground};
use neopets_data::model::ModelData;
use neopets_data::{Disc, resource};
use neopets_sim::contact::{self, BoxShape, WallEdge};
use neopets_sim::hero::{HeroInput, Locomotion};

pub struct Play {
    pub loco: Locomotion,
    pub hero: HeroAnim,
    pub model: ModelData,
    ground: Option<Ground>,
    /// Rectangle primitives that push characters out (SPEC-0018 box push; new-game mask 0).
    boxes: Vec<BoxShape>,
    root_step: [f32; 3],
    /// Set once the hero's meshes exist.
    pub spawned: bool,
}

impl Play {
    /// Load a hero outfit and a world pack's ground and spawn point.
    pub fn load(disc: &mut Disc, index: &ArchiveIndex, pack: u32, outfit: u32) -> Result<Self, String> {
        let model = crate::archive_view::model(disc, index, outfit)?.ok_or("Outfit has no model")?;
        let hero = crate::archive_view::hero(disc, index, outfit)?.ok_or("Not a hero outfit (504–519)")?;
        let bytes = index.read(disc, pack)?;
        let chain = resource::walk_chunks(&bytes, None)?;
        let ground = Ground::of(&bytes, &chain.chunks, CellOrder::XMajor);
        let (spawn, facing) = ground::spawn(&bytes, &chain.chunks).unwrap_or(([0.0; 3], [1.0, 0.0, 0.0]));
        let y = ground.as_ref().map_or(spawn[1], |g| ground::spawn_height(g, spawn));
        let boxes = ground::rectangles(&bytes, &chain.chunks)
            .into_iter()
            .filter(|r| r.pushes(0))
            .map(|r| BoxShape { origin: r.origin, axes: r.axes, extents: r.extents })
            .collect();
        Ok(Self {
            loco: Locomotion::spawn([spawn[0], y, spawn[2]], facing),
            hero,
            model,
            ground,
            boxes,
            root_step: [0.0; 3],
            spawned: false,
        })
    }

    /// One game tick in the hero order of SPEC-0040 Behavior 3: the hero update (stick, turn,
    /// tiers), travel by the root step stored on the previous tick, then the animation pass,
    /// which stores the next root step.
    pub fn tick(&mut self, input: &HeroInput, camera_forward: [f32; 3], camera_left: [f32; 3]) {
        self.loco.update(input, camera_forward, camera_left);
        self.hero.play(self.loco.action());
        self.hero.set_rate(self.loco.rate);
        let ground = self.ground.as_ref();
        self.loco.travel(self.root_step, |x, z, c| {
            ground.map_or(ground::MISS, |g| g.probe(x, z, c))
        });
        // SPEC-0040 Behavior 14A driver branch 0, in order: the cell's frame records (walls),
        // then the box primitives, each with the hero radius. Not implemented: cell lists A and
        // B as such, cylinders, spheres, the ledge tests, the slide response and recovery.
        if let Some(g) = ground {
            let p = self.loco.position;
            let walls: Vec<WallEdge> = g
                .frames_at(p[0], p[2])
                .into_iter()
                .map(|f| WallEdge { face: f.face, run: f.run, top: f.top, length: f.length, height: f.height })
                .collect();
            contact::resolve_walls(&mut self.loco.position, &walls, contact::HERO_RADIUS);
        }
        contact::resolve(&mut self.loco.position, &self.boxes, contact::HERO_RADIUS);
        self.root_step = self.hero.tick();
    }

    pub fn status(&self) -> String {
        let p = self.loco.position;
        format!(
            "PLAY (M7 groundwork): hero at ({:.1}, {:.1}, {:.1}), state {}, action {}, rate {:.2}. \
             WASD move, Shift run (R1), drag to turn the camera. Frame-record and box walls; no gravity or jumps yet",
            p[0], p[1], p[2], self.loco.state, self.loco.action(), self.loco.rate
        )
    }
}
