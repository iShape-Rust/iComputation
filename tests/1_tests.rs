#[cfg(test)]
mod tests {
    use bytemuck::{Pod, Zeroable};
    use rand::Rng;
    use i_computation::buffer::BufferMode;
    use i_computation::context::GpuContext;
    use i_computation::solution::WorkGroup;
    use i_computation::solver::SolverBuilder;

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    struct IntPoint {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Pod, Zeroable)]
    struct Params {
        radius: i32,
    }

    #[derive(Debug, Clone, Copy)]
    struct Rect {
        min_x: i32,
        max_x: i32,
        min_y: i32,
        max_y: i32,
    }

    impl IntPoint {
        fn random(rect: Rect, count: usize) -> Vec<IntPoint> {
            let mut points = Vec::with_capacity(count);
            let mut rng = rand::thread_rng();
            for _ in 0..count {
                let x = rng.gen_range(rect.min_x..=rect.max_x);
                let y = rng.gen_range(rect.min_y..=rect.max_y);
                points.push(IntPoint { x, y });
            }
            points
        }
    }

    #[test]
    fn test_counter() {
        let context = GpuContext::new_sync();
        let solver = SolverBuilder::new()
            .set_shader(include_str!("shader_1.wgsl"), "main")
            .add_storage(BufferMode::Read)
            .add_storage(BufferMode::Read)
            .add_uniform()
            .add_storage(BufferMode::Write)
            .build(&context);

        let rect = Rect {
            min_x: 0,
            max_x: 10_000,
            min_y: 0,
            max_y: 10_000,
        };

        let points = IntPoint::random(rect, 1_000_000);
        let stars = IntPoint::random(rect, 100);
        let params = Params { radius: 5 };
        // let result_size = stars.len() * std::mem::size_of::<u32>();

        let mut solution = solver.solution();
        solution.bind_data_buffer(0, &points, &context);
        solution.bind_data_buffer(1, &stars, &context);
        solution.bind_data_buffer(2, &[params], &context);
        solution.bind_size_buffer::<u32>(3, stars.len(), &context);

        solution.execute(WorkGroup::new(stars.len()), &context);
        let output: Vec<u32> = solution.read_sync(3, &context);

        println!("output: {:?}", &output);
    }
}