const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
let width, height;
function resize(){ width = canvas.width = window.innerWidth; height = canvas.height = window.innerHeight; }
window.addEventListener('resize', resize); resize();


// Timeline state
let t = 0;
let playing = false;
let speed = 0.2;
const playPauseBtn = document.getElementById('playPause');
const timelineSlider = document.getElementById('timeline');
const timeLabel = document.getElementById('timeLabel');
playPauseBtn.onclick = ()=>{ playing=!playing; playPauseBtn.textContent=playing?'Pause':'Play'; };
timelineSlider.oninput = (e)=>{ t=parseFloat(e.target.value); timeLabel.textContent=`t = ${t.toFixed(2)}`; };


// Define Rothko blocks
const blocks = [
{color:[0.8,0.1,0.1], opacity:0.7, height:0.25, phase:0},
{color:[0.9,0.6,0.2], opacity:0.6, height:0.25, phase:0.5},
{color:[0.2,0.2,0.7], opacity:0.5, height:0.25, phase:1.0},
{color:[0.1,0.6,0.2], opacity:0.6, height:0.25, phase:1.5}
];


function draw(){
ctx.clearRect(0,0,width,height);
let y=0;
for(let b of blocks){
// Animate opacity with subtle sine wave
const a = b.opacity + 0.1*Math.sin(2*Math.PI*(t+b.phase));
ctx.fillStyle = `rgba(${b.color.map(c=>Math.floor(c*255)).join(',')},${a})`;
const h = b.height*height;
ctx.fillRect(0,y,width,h);
y += h;
}
}


function animate(){
if(playing){
t += speed * 0.016;
if(t>1) t=0;
timelineSlider.value=t;
timeLabel.textContent=`t = ${t.toFixed(2)}`;
}
draw();
requestAnimationFrame(animate);
}


requestAnimationFrame(animate);