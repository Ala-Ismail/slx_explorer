use std::collections::BTreeMap;

use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};
use slx_viewer::model::{Block, Bounds, Endpoint, Port, Segment, System};

pub struct Scene {
    pub blocks: Vec<Option<Rect>>,
    pub nets: Vec<SignalNet>,
    pub ports: Vec<Pos2>,
    pub labels: Vec<(Pos2, String)>,
    pub operators: Vec<(Pos2, String)>,
    pub bounds: Option<Rect>,
}

pub struct SignalNet {
    pub paths: Vec<WirePath>,
    pub source: Option<Pos2>,
    pub destinations: Vec<Option<Pos2>>,
}

pub struct WirePath {
    pub points: Vec<Pos2>,
    pub bounds: Rect,
}

impl Scene {
    pub fn new(system: &System) -> Self {
        let block_rects: Vec<_> = system
            .blocks
            .iter()
            .map(|block| block.bounds.and_then(rect))
            .collect();
        let mut scene = Self {
            blocks: Vec::new(),
            nets: Vec::with_capacity(system.connections.len()),
            ports: Vec::new(),
            labels: Vec::new(),
            operators: Vec::new(),
            bounds: None,
        };
        let counts: Vec<_> = system
            .blocks
            .iter()
            .map(|block| {
                let mut counts: BTreeMap<_, _> = block
                    .port_counts
                    .iter()
                    .map(|(kind, count)| (kind.as_str(), *count))
                    .collect();
                for port in &block.ports {
                    let count = counts.entry(&port.kind).or_default();
                    *count = (*count).max(port.index);
                }
                counts
            })
            .collect();
        for (index, (block, bounds)) in system.blocks.iter().zip(&block_rects).enumerate() {
            if let Some(bounds) = bounds {
                scene.include(*bounds);
                scene.add_label(bounds.center_bottom() + vec2(0.0, 3.0), &block.name);
                if matches!(block.block_type.as_str(), "Sum" | "Product")
                    && let Some(inputs) = block.properties.get("Inputs")
                {
                    let default = if block.block_type == "Sum" { '+' } else { '*' };
                    let signs: Vec<char> = if let Ok(count) = inputs.trim().parse::<usize>() {
                        vec![default; count.min(4096)]
                    } else {
                        inputs
                            .chars()
                            .filter(|c| matches!(c, '+' | '-' | '*' | '/'))
                            .take(4096)
                            .collect()
                    };
                    for (input, sign) in signs.iter().enumerate() {
                        let port = Port {
                            kind: "in".into(),
                            index: input as u32 + 1,
                        };
                        let mut input_counts = counts[index].clone();
                        input_counts.insert("in", signs.len() as u32);
                        let anchor = port_anchor(block, *bounds, &port, &input_counts).at;
                        let inward = (bounds.center() - anchor).normalized() * 7.0;
                        scene.operators.push((anchor + inward, sign.to_string()));
                    }
                }
                for port in &block.ports {
                    scene
                        .ports
                        .push(port_anchor(block, *bounds, port, &counts[index]).at);
                }
            }
        }
        let anchor = |endpoint: &Endpoint| {
            let index = endpoint.block?;
            Some(port_anchor(
                &system.blocks[index],
                block_rects[index]?,
                endpoint.port.as_ref()?,
                &counts[index],
            ))
        };
        for line in &system.connections {
            let source = line.source.as_ref().and_then(anchor);
            let mut paths = Vec::new();
            if let Some(start) = source {
                scene.add_segment(&line.route, start.at, Some(start), &anchor, &mut paths);
            }
            scene.nets.push(SignalNet {
                paths,
                source: source.map(|port| port.at),
                destinations: signal_segments(&line.route)
                    .filter_map(|segment| segment.destination.as_ref())
                    .map(|endpoint| anchor(endpoint).map(|port| port.at))
                    .collect(),
            });
        }
        for annotation in &system.annotations {
            match *annotation.position.as_slice() {
                [x, y] => {
                    if let Some(anchor) = point([x, y]) {
                        scene.add_label(anchor, &annotation.text);
                    }
                }
                [left, top, right, bottom] => {
                    if let Some(bounds) = rect(Bounds {
                        min: [left, top],
                        max: [right, bottom],
                    }) {
                        scene.include(bounds);
                        scene.add_label(bounds.center_top(), &annotation.text);
                    }
                }
                _ => {}
            }
        }
        scene.blocks = block_rects;
        scene
    }

    pub fn hit_net(&self, point: Pos2, scale: f32) -> Option<usize> {
        let tolerance = 5.0 / scale;
        let mut nearest = tolerance * tolerance;
        let mut hit = None;
        for (index, net) in self.nets.iter().enumerate() {
            for path in &net.paths {
                if !path.bounds.expand(tolerance).contains(point) {
                    continue;
                }
                for pair in path.points.windows(2) {
                    let distance = segment_distance_sq(point, pair[0], pair[1]);
                    if distance <= nearest {
                        nearest = distance;
                        hit = Some(index);
                    }
                }
            }
        }
        hit
    }

    fn include(&mut self, rect: Rect) {
        self.bounds = Some(self.bounds.map_or(rect, |bounds| bounds.union(rect)));
    }

    fn add_label(&mut self, at: Pos2, text: &str) {
        if !text.is_empty() {
            self.labels.push((at, text.into()));
        }
    }

    fn add_segment(
        &mut self,
        segment: &Segment,
        start: Pos2,
        source: Option<PortAnchor>,
        anchor: &impl Fn(&Endpoint) -> Option<PortAnchor>,
        paths: &mut Vec<WirePath>,
    ) {
        let mut points = vec![start];
        let mut junction = start;
        for &offset in &segment.offsets {
            let Some(offset) = point(offset) else { return };
            let Some(next) = point([
                f64::from(junction.x + offset.x),
                f64::from(junction.y + offset.y),
            ]) else {
                return;
            };
            junction = next;
            points.push(junction);
        }
        if let Some(port) = source.filter(|_| points.len() > 1) {
            let first = points[1..]
                .iter()
                .position(|&point| !port.block.contains(point))
                .map_or(points.len() - 1, |index| index + 1);
            if first != 1 || !port.faces(points[first]) {
                let mut exit = vec![points[first]];
                port.attach(&mut exit);
                exit.reverse();
                exit.extend_from_slice(&points[first + 1..]);
                points = exit;
            }
        }
        if let Some(port) = segment.destination.as_ref().and_then(anchor) {
            // Saved branches still start at the original junction, even if the
            // estimated port makes part of a leaf route fall inside its block.
            if segment.branches.is_empty() && junction != port.at {
                while points.len() > 1 && port.block.contains(*points.last().unwrap()) {
                    points.pop();
                }
            }
            if let Some(source) = source.filter(|_| points.len() == 1)
                && !(source.faces(port.at) && port.faces(source.at))
            {
                let distance = (port.at - source.at).dot(source.outward);
                let lead = if distance > 0.0 {
                    (distance / 2.0).min(PORT_CLEARANCE)
                } else {
                    PORT_CLEARANCE
                };
                push_point(&mut points, source.at + source.outward * lead);
            }
            port.attach(&mut points);
        }
        if points.len() > 1 {
            let bounds = Rect::from_points(&points);
            self.include(bounds);
            paths.push(WirePath { points, bounds });
        }
        if let Some(label) = &segment.label {
            self.add_label(start.lerp(junction, 0.5), label);
        }
        for branch in &segment.branches {
            self.add_segment(branch, junction, None, anchor, paths);
        }
    }
}

pub(super) fn signal_segments(route: &Segment) -> impl Iterator<Item = &Segment> {
    let mut pending = vec![route];
    std::iter::from_fn(move || {
        let segment = pending.pop()?;
        pending.extend(segment.branches.iter().rev());
        Some(segment)
    })
}

pub(super) fn segment_distance_sq(point: Pos2, start: Pos2, end: Pos2) -> f32 {
    let delta = end - start;
    let length = delta.length_sq();
    if length == 0.0 {
        return point.distance_sq(start);
    }
    let fraction = ((point - start).dot(delta) / length).clamp(0.0, 1.0);
    point.distance_sq(start + delta * fraction)
}

pub(super) fn point([x, y]: [f64; 2]) -> Option<Pos2> {
    const RANGE: std::ops::RangeInclusive<f64> = -10_000_000.0..=10_000_000.0;
    (RANGE.contains(&x) && RANGE.contains(&y)).then(|| pos2(x as f32, y as f32))
}

pub(super) fn rect(bounds: Bounds) -> Option<Rect> {
    Some(Rect::from_two_pos(point(bounds.min)?, point(bounds.max)?))
}

const PORT_CLEARANCE: f32 = 8.0;

#[derive(Clone, Copy)]
struct PortAnchor {
    at: Pos2,
    outward: Vec2,
    block: Rect,
}

impl PortAnchor {
    fn faces(self, point: Pos2) -> bool {
        let offset = point - self.at;
        let distance = offset.dot(self.outward);
        distance > 0.0 && offset == self.outward * distance
    }

    fn attach(self, path: &mut Vec<Pos2>) {
        let waypoint = *path.last().unwrap();
        if waypoint == self.at {
            return;
        }
        let distance = (waypoint - self.at).dot(self.outward);
        if distance > 0.0 {
            push_point(path, self.at + self.outward * distance);
        } else {
            let tangent = vec2(-self.outward.y, self.outward.x);
            let extent = self.block.size().dot(tangent.abs()) / 2.0;
            let offset = (waypoint - self.block.center()).dot(tangent);
            let clearance = extent + PORT_CLEARANCE;
            let side = if offset < 0.0 {
                offset.min(-clearance)
            } else {
                offset.max(clearance)
            };
            let outside = waypoint + tangent * (side - offset);
            push_point(path, outside);
            push_point(path, outside + self.outward * (PORT_CLEARANCE - distance));
            push_point(path, self.at + self.outward * PORT_CLEARANCE);
        }
        push_point(path, self.at);
    }
}

fn push_point(points: &mut Vec<Pos2>, point: Pos2) {
    if points.last() != Some(&point) {
        points.push(point);
    }
}

fn port_anchor(
    block: &Block,
    bounds: Rect,
    port: &Port,
    counts: &BTreeMap<&str, u32>,
) -> PortAnchor {
    let count = counts
        .get(port.kind.as_str())
        .copied()
        .unwrap_or(port.index);
    let position = port.index as f32 / (count as f32 + 1.0);
    let (mut x, y, mut outward) = match port.kind.as_str() {
        "in" | "lconn" => (0.0, position, -Vec2::X),
        "out" | "rconn" => (1.0, position, Vec2::X),
        "state" => (position, 1.0, Vec2::Y),
        _ => (position, 0.0, -Vec2::Y),
    };
    if block.mirrored {
        x = 1.0 - x;
        outward.x = -outward.x;
    }
    let (x, y, outward) = match block.rotation_degrees {
        90 => (1.0 - y, x, vec2(-outward.y, outward.x)),
        180 => (1.0 - x, 1.0 - y, -outward),
        270 => (y, 1.0 - x, vec2(outward.y, -outward.x)),
        _ => (x, y, outward),
    };
    PortAnchor {
        at: pos2(
            bounds.min.x + bounds.width() * x,
            bounds.min.y + bounds.height() * y,
        ),
        outward,
        block: bounds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slx_viewer::model::Connection;
    use slx_viewer::{Limits, load};

    #[test]
    fn saved_branch_offsets_share_the_parent_junction() {
        let model = load(
            "tests/fixtures/dc_motor_speed_control.slx",
            &Limits::default(),
        )
        .unwrap();
        let scene = Scene::new(&model.systems[model.root]);
        let index = scene.hit_net(pos2(1110.0, 425.0), 1.0).unwrap();
        let net = &scene.nets[index];
        for path in [
            &[pos2(1090.0, 425.0), pos2(1130.0, 425.0)][..],
            &[pos2(1130.0, 425.0), pos2(1132.0, 425.0)],
            &[
                pos2(1130.0, 425.0),
                pos2(1130.0, 570.0),
                pos2(1160.0, 570.0),
            ],
        ] {
            assert!(net.paths.iter().any(|wire| wire.points == path), "{path:?}");
        }
        assert_eq!(scene.hit_net(pos2(1130.0, 510.0), 1.0), Some(index));
        assert_eq!(net.source, Some(pos2(1090.0, 425.0)));
        assert_eq!(net.destinations.len(), 3);
        assert_eq!(net.destinations[2], Some(pos2(1160.0, 570.0)));
    }

    #[test]
    fn crossing_nets_use_nearest_path_and_screen_space_tolerance() {
        let mut scene = Scene::new(&System {
            name: String::new(),
            parent: None,
            source_part: String::new(),
            blocks: Vec::new(),
            connections: Vec::new(),
            annotations: Vec::new(),
        });
        scene.nets = [
            [pos2(-100.0, 0.0), pos2(100.0, 0.0)],
            [pos2(0.0, -100.0), pos2(0.0, 100.0)],
        ]
        .map(|points| SignalNet {
            paths: vec![WirePath {
                bounds: Rect::from_points(&points),
                points: points.into(),
            }],
            source: None,
            destinations: Vec::new(),
        })
        .into();

        assert_eq!(scene.hit_net(pos2(4.0, 2.0), 1.0), Some(0));
        assert_eq!(scene.hit_net(pos2(2.0, 4.0), 1.0), Some(1));
        assert_eq!(scene.hit_net(Pos2::ZERO, 1.0), Some(1));
        for zoom in [0.1, 1.0, 8.0] {
            assert_eq!(scene.hit_net(pos2(80.0, 4.0 / zoom), zoom), Some(0));
            assert_eq!(scene.hit_net(pos2(80.0, 6.0 / zoom), zoom), None);
        }
    }

    #[test]
    fn rotated_and_mirrored_endpoints_attach_from_the_correct_sides() {
        let mut model = load(
            "tests/fixtures/dc_motor_speed_control.slx",
            &Limits::default(),
        )
        .unwrap();
        let system = &mut model.systems[model.root];
        system.blocks.truncate(2);
        let endpoint = |block, kind: &str| Endpoint {
            raw: format!("{block}#{kind}:1"),
            block: Some(block),
            port: Some(Port {
                kind: kind.into(),
                index: 1,
            }),
        };
        let source = endpoint(0, "out");
        let destination = endpoint(1, "in");
        for (index, block) in system.blocks.iter_mut().enumerate() {
            block.port_counts.clear();
            block.ports = vec![
                if index == 0 { &source } else { &destination }
                    .port
                    .clone()
                    .unwrap(),
            ];
        }
        system.blocks[0].bounds = Some(Bounds {
            min: [0.0, 0.0],
            max: [20.0, 20.0],
        });
        system.blocks[0].rotation_degrees = 90;
        system.blocks[0].mirrored = false;
        system.blocks[1].bounds = Some(Bounds {
            min: [60.0, 50.0],
            max: [80.0, 70.0],
        });
        system.blocks[1].rotation_degrees = 0;
        system.blocks[1].mirrored = true;
        system.connections = vec![Connection {
            source: Some(source),
            route: Segment {
                destination: Some(destination),
                offsets: vec![[-20.0, 0.0], [0.0, 10.0], [60.0, 0.0]],
                label: None,
                branches: Vec::new(),
            },
        }];

        let scene = Scene::new(system);
        let wire = &scene.nets[0].paths[0].points;
        assert_eq!(scene.ports, [pos2(10.0, 20.0), pos2(80.0, 60.0)]);
        assert!(wire.starts_with(&[pos2(10.0, 20.0), pos2(10.0, 28.0)]));
        assert!(
            wire.windows(3)
                .any(|points| points == [pos2(-10.0, 20.0), pos2(-10.0, 30.0), pos2(50.0, 30.0)])
        );
        assert!(wire.ends_with(&[pos2(88.0, 30.0), pos2(88.0, 60.0), pos2(80.0, 60.0)]));
        assert!(
            wire.windows(2)
                .all(|pair| pair[0].x == pair[1].x || pair[0].y == pair[1].y)
        );
    }
}
