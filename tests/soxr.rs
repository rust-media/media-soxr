use media_soxr::*;

#[test]
fn test_version() {
    let version = Soxr::<Packed<f32>, Packed<f32>>::version();
    println!("soxr version: {}", version);
    assert!(!version.is_empty());
}

#[test]
fn test_new() {
    let soxr = Soxr::<Packed<f32>, Packed<f32>>::new(44100.0, 48000.0, 2, None, None);

    assert!(soxr.is_ok());

    let soxr = soxr.unwrap();

    println!("soxr engine: {:?}", soxr.engine());
}

#[test]
fn test_process() {
    let mut soxr = Soxr::<Planar<f32>, Packed<i16>>::new(44100.0, 48000.0, 2, None, None).unwrap();

    let input: &[&[f32]] = &[&vec![0.0; 44100], &vec![0.0; 44100]];
    let mut output: Vec<i16> = vec![0; 48000 * 2];

    let input_buffer = SampleBuffer::Planar(input);
    let output_buffer = SampleBufferMut::Packed(&mut output);

    let result = soxr.process(Some(input_buffer), output_buffer);

    assert!(result.is_ok());

    let (idone, odone) = result.unwrap();

    println!("input done: {}", idone);
    println!("output done: {}", odone);

    assert_eq!(idone, 44100);
}
