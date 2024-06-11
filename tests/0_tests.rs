#[cfg(test)]
mod tests {
    use i_computation::buffer::BufferMode;
    use i_computation::context::GpuContext;
    use i_computation::solution::WorkGroup;
    use i_computation::solver::SolverBuilder;

    #[test]
    fn test_hello_world() {
        let context = GpuContext::new_sync();
        let solver = SolverBuilder::new()
            .set_shader(include_str!("shader_0.wgsl"), "main")
            .add_storage(BufferMode::ReadWrite)
            .build(&context);

        let input = vec![1.0f32, 2.0f32];
        let mut solution = solver.solution();
        solution.bind_data_buffer(0, &input, &context);
        solution.execute(WorkGroup::new(input.len()), &context);
        let output: Vec<f32> = solution.read_sync(0, &context);

        println!("output: {:?}", &output);
    }
}