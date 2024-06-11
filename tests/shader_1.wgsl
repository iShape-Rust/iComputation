struct IntPoint {
    x: i32,
    y: i32,
};

struct Points {
    data: array<IntPoint>,
};

struct Stars {
    data: array<IntPoint>,
};

struct Params {
    radius: i32,
};

struct Counts {
    data: array<u32>,
};

@group(0)
@binding(0)
var<storage, read> points: Points;

@group(0)
@binding(1)
var<storage, read> stars: Stars;

@group(0)
@binding(2)
var<uniform> params: Params;

@group(0)
@binding(3)
var<storage, read_write> counts: Counts;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let star_idx = global_id.x;
    let star = stars.data[star_idx];
    let radius = params.radius;

    var count: u32 = 0;
    for (var i = 0u; i < arrayLength(&points.data); i = i + 1u) {
        let p = points.data[i];
        let dx = p.x - star.x;
        let dy = p.y - star.y;
        if (dx * dx + dy * dy <= radius * radius) {
            count = count + 1;
        }
    }
    counts.data[star_idx] = count;
}
