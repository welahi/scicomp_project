

use std::os::unix::raw::dev_t;

use wgpu::{BindGroup, BindGroupLayout, ShaderModel, ShaderModule};

use crate::*;

pub struct GPUSparseMultiplyer {
    pub wgpu_task: WgpuTask,
    pub a: CSR,
    pub b: CSR,
    pub shader: ShaderModule,
    pub batch_size: usize,
    pub bind_groups: Option<(BindGroup, BindGroup, BindGroup)>,
    pub bind_group_layouts: Option<(BindGroupLayout, BindGroupLayout, BindGroupLayout)>,
    pub buffer_res: Option<ResultBuffer>,
    pub buffer_res_staging: Option<ResultBuffer>

}



impl GPUSparseMultiplyer {
    pub async fn new(a: CSR, b: CSR, batch_size: usize) -> Self {
        let wgpu_task = WgpuTask::new().await;

        let device = &wgpu_task.device;

        // Load Shader

        let shader_code = std::fs::read_to_string("shader/sparse_mul.wgsl").expect("Shader file not found.");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sparse Matrix Multiplication Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });



        
        GPUSparseMultiplyer{wgpu_task, a, b, shader, batch_size, bind_groups: None, bind_group_layouts: None, buffer_res: None, buffer_res_staging: None}
    }


    pub fn create_and_load_buffer(&mut self) {
        let device = &self.wgpu_task.device;

        // Create Buffer for CSR Matrix Data 

        let buffer_a = CSRBuffer::new(&device, &self.a, "A", wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC);
        let buffer_b = CSRBuffer::new(&device, &self.b, "B", wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC);

        let nnz_pred = size_prediction(&self.a, &self.b);

        let buffer_res = ResultBuffer::new(&device, nnz_pred, self.b.shape.1, self.batch_size, "C", wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC);
        let buffer_res_staging = ResultBuffer::new(&device, nnz_pred, self.b.shape.1, self.batch_size, "C", wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST);



        // Bind group

        let bg_a_entries = buffer_a.gen_bind_group_entries(0, true);
        let bg_b_entries = buffer_b.gen_bind_group_entries(0, true);

        let bg_res_entries = buffer_res.gen_bind_group_entries(0, false);


        let bg_a_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout: Matrix A"),
            entries: &bg_a_entries
        });  
        let bg_b_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout: Matrix B"),
            entries: &bg_b_entries
        });  
        let bg_res_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout: Result"),
            entries: &bg_res_entries
        });

        let bg_a = buffer_a.create_bind_group(&device, &bg_a_layout);
        let bg_b = buffer_b.create_bind_group(&device, &bg_b_layout);
        let bg_res = buffer_res.create_bind_group(&device, &bg_res_layout);


        self.bind_groups = Some((bg_a, bg_b, bg_res));
        self.bind_group_layouts = Some((bg_a_layout, bg_b_layout, bg_res_layout));
        self.buffer_res_staging = Some(buffer_res_staging);
        self.buffer_res = Some(buffer_res);



    }


    // pub async fn doit(&self) -> (usize, GlobDataEntry) {
    pub async fn doit(&self) -> COO {
        let msg = "No Bind group (layout) found";

        let device = &self.wgpu_task.device;

        // Pipeline

        let pipeline_layout = self.wgpu_task.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&self.bind_group_layouts.as_ref().expect(msg).0, &self.bind_group_layouts.as_ref().expect(msg).1, &self.bind_group_layouts.as_ref().expect(msg).2],
            push_constant_ranges: &[],
        });

        let pipeline = self.wgpu_task.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &self.shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });


        // Encode / Queue / Compute pass

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Compute Encoder") });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &self.bind_groups.as_ref().expect(msg).0, &[]);
            compute_pass.set_bind_group(1, &self.bind_groups.as_ref().expect(msg).1, &[]);
            compute_pass.set_bind_group(2, &self.bind_groups.as_ref().expect(msg).2, &[]);
            compute_pass.dispatch_workgroups(1,1, 1); // Angepasste Dispatch-Parameter
        }

        let nnz_pred = size_prediction(&self.a, &self.b);

        let buffer_res_staging = self.buffer_res_staging.as_ref().expect(msg);

        self.buffer_res.as_ref().expect(msg).copy_b2b(&buffer_res_staging, nnz_pred, self.b.shape.1, self.batch_size,&mut encoder);
        self.wgpu_task.queue.submit(Some(encoder.finish()));

        let result_buffer_idx = buffer_res_staging.idx.slice(..);
        let result_buffer_glob_data = buffer_res_staging.glob_data.slice(..);


        let (sender, receiver_idx) = oneshot_channel();
        unsafe {
            result_buffer_idx.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        } 
        

        let (sender, receiver_data) = oneshot_channel();
        unsafe {        
            result_buffer_glob_data.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        }


        device.poll(wgpu::Maintain::Wait);
        receiver_idx.receive().await.unwrap().unwrap();
        receiver_data.receive().await.unwrap().unwrap();

        let data_result_idx = result_buffer_idx.get_mapped_range();
        let result: &[u32] = bytemuck::cast_slice(&data_result_idx);
        let n_c_data = result[0] as usize -1;
        // println!("Ergebnis-Matrix: {:?}", n_c_data);


        let data_result_idx = result_buffer_glob_data.get_mapped_range();
        let result: &[GlobDataEntry] = bytemuck::cast_slice(&data_result_idx);

        // let results = Vec::from(result);
        // let gd = result[0];

        let data_final: Vec<(usize, usize, f64)> = result[..n_c_data].into_iter().map(|entry| (entry.i as usize, entry.j as usize, entry.x as f64)).collect();

        COO {data: data_final, shape: (self.a.shape.0, self.b.shape.1)}

        // (n_c_data as usize, gd)

    }

}
