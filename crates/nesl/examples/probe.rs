fn main() {
    // count main-loop iterations per frame: a game that cannot keep up runs in slow motion
    let src = std::fs::read_to_string("examples/quest.nesl").unwrap()
        .replace("var cam: u16;", "var cam: u16; var ticks: u16;")
        .replace("        let pad = joypad(0);\n        let pressed", "        ticks += 1;\n        let pad = joypad(0);\n        let pressed");
    let out = nesl::compile(&src).unwrap();
    let v = |n: &str| out.vars[n].addr;
    let rd16 = |nes: &nes_sim::TestNes, n: &str| nes.peek(v(n)) as u16 | (nes.peek(v(n) + 1) as u16) << 8;
    let mut nes = nes_sim::TestNes::from_ines(&out.rom).unwrap();
    nes.run_frames(10).unwrap();
    nes.set_buttons(0, 0x10); nes.run_frames(2).unwrap(); nes.set_buttons(0, 0); nes.run_frames(10).unwrap();
    let (t0, f0) = (rd16(&nes, "ticks"), nes.frames);
    for f in 0..600u32 {
        nes.set_buttons(0, if f % 24 < 3 { 0x81 } else { 0x01 });
        let before = (nes.peek(v("lives")), rd16(&nes, "hero.x"), nes.peek(v("hero.y")), nes.peek(v("hurt")));
        nes.run_frames(1).unwrap();
        if nes.peek(v("lives")) != before.0 {
            println!("frame {f}: life lost at hero.x={} y={} (hurt was {})", before.1, before.2, before.3);
        }
    }
    let ticks = rd16(&nes, "ticks") - t0;
    println!("{ticks} loop iterations in {} frames", nes.frames - f0);
}
