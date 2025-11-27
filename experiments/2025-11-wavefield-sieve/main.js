// main.js — glue: Three.js setup, CPU sieve, animation loop, uploads
if (timelineSlider) {
timelineSlider.oninput = (e) => {
t = parseFloat(e.target.value);
timeLabel.textContent = `t = ${t.toFixed(2)}`;
};
}


function animate(){
const t = performance.now()/1000.0;
const time = (t - t0) * speed;
uniforms.u_time.value = time;


// for each prime event, compute front and mark multiples
const v = 6.0; // speed, should match shader
for(let i=0;i<Math.min(primes.length,128);i++){
const p = primes[i];
const spawn = i * STEP;
const dt = time - spawn;
if(dt < 0) continue;
const front = Math.round(v * dt);
// mark any multiples near front as crossed
multiples[i].forEach(m => {
if(Math.abs(m - front) <= 1) stateCrossed[m] = 1;
});
}


updateCrossedTexture();


renderer.setSize(window.innerWidth, window.innerHeight, false);
renderer.render(scene, camera);
if(!paused) requestAnimationFrame(animate);
}


btnRestart.onclick = ()=>{ setup(); };
btnSpeed.onclick = ()=>{ speed = (speed===1.0)?2.0:1.0 };


inputN.onchange = ()=>{ setup(); };
inputStep.onchange = ()=>{ STEP = parseFloat(inputStep.value); setup(); };
inputPrimeCap.onchange = ()=>{ setup(); };


// init + run
setup();
requestAnimationFrame(animate);


// handle resize
window.addEventListener('resize', ()=>{
renderer.setSize(window.innerWidth, window.innerHeight, false);
});
});
})();


(function(){ // existing setup ...

// --- Timeline State --- let t = 0; let playing = false; let speed = 0.2; // multiplier

const playPauseBtn = document.getElementById('playPause'); const timelineSlider = document.getElementById('timeline'); const timeLabel = document.getElementById('timeLabel');

if(playPauseBtn){ playPauseBtn.onclick = () => { playing = !playing; playPauseBtn.textContent = playing ? 'Pause' : 'Play'; }; }

if(timelineSlider){ timelineSlider.oninput = (e) => { t = parseFloat(e.target.value); timeLabel.textContent = t = ${t.toFixed(2)}; }; }

// Animation loop function animate(){ if(playing){ t += speed * 0.016; if(t > 1) t = 0; if(timelineSlider) timelineSlider.value = t; if(timeLabel) timeLabel.textContent = t = ${t.toFixed(2)}; }
// update shader uniform
if(mesh.material.uniforms && mesh.material.uniforms.u_time){
mesh.material.uniforms.u_time.value = t;
}


renderer.setSize(window.innerWidth, window.innerHeight, false);
renderer.render(scene, camera);
requestAnimationFrame(animate);
}

requestAnimationFrame(animate); })();