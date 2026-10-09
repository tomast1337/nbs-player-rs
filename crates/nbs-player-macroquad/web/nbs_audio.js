// miniquad plugin: Web Audio output for the core software mixer.
// Rust exports `nbs_audio_fill(frames) -> *const f32` (interleaved stereo); a
// ScriptProcessorNode pulls it. Loaded after mq_js_bundle.js, before load().
(function () {
    "use strict";
    let ctx = null;

    function audio_init() {
        const AC = window.AudioContext || window.webkitAudioContext;
        ctx = new AC({ latencyHint: "interactive" });

        const node = ctx.createScriptProcessor(2048, 0, 2);
        node.onaudioprocess = function (e) {
            const out = e.outputBuffer;
            const n = out.length;
            const ptr = wasm_exports.nbs_audio_fill(n);
            const pcm = new Float32Array(wasm_memory.buffer, ptr, n * 2);
            const l = out.getChannelData(0);
            const r = out.getChannelData(1);
            for (let i = 0; i < n; i++) {
                l[i] = pcm[2 * i];
                r[i] = pcm[2 * i + 1];
            }
        };
        node.connect(ctx.destination);

        // Browsers keep the context suspended until a user gesture.
        const resume = function () {
            if (ctx.state !== "running") ctx.resume();
        };
        ["pointerdown", "keydown", "touchend"].forEach(function (ev) {
            window.addEventListener(ev, resume, { capture: true });
        });
        return ctx.sampleRate;
    }

    miniquad_add_plugin({
        register_plugin: function (importObject) {
            importObject.env.nbs_audio_init = audio_init;
        },
        version: 1,
        name: "nbs_audio",
    });
})();
