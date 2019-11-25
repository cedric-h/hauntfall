// remove padding from sides of site, to make room for canvas
document.body.style.margin = "0px";
document.body.style.padding = "0px";
document.body.style.overflow = "hidden";

const renderer = new THREE.WebGLRenderer( { antialias: true } );
//renderer.gammaOutput = true;
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.setClearColor(new THREE.Color(0.0285, 0.024, 0.054), 1.0);

// where all of the things to render are stored
const scene = new THREE.Scene();

// camera setup
const camera = new THREE.PerspectiveCamera(39.6, window.innerWidth / window.innerHeight, 0.01, 100);
camera.position.y = 20;
camera.position.x = 15;
camera.position.z = 15;
camera.lookAt(0, 0, 0);

/*
const composer = new THREE.EffectComposer( renderer );
const ssaoPass = new THREE.SSAOPass( scene, camera, window.innerWidth, window.innerHeight );
ssaoPass.kernelRadius = 128;
ssaoPass.output = parseInt(THREE.SSAOPass.OUTPUT.Beauty);
composer.addPass( ssaoPass );
*/

const light = new THREE.AmbientLight(0x37324C, 1.63)
scene.add(light);

// map of appearance name -> mesh
const appearance_names = [
	"StoneOutcroppingFloorRight",
	"StoneOutcroppingFloorLeft",
	"StoneOutcroppingFloorBottom",
	"StoneOutcroppingFloorCorner",
	"Lantern",
	"Skeleton",
	"StoneWall",
];
const appearances = {};
const loader = new THREE.GLTFLoader().setPath("assets/");
for (const name of appearance_names) {
	loader.load(name + ".glb", (gltf) => {
		//console.log("loaded one!", gltf);

		gltf.scene.traverse((obj) => {
			if (obj.type == "SpotLight") {
				obj.intensity /= 225;
				obj.distance = 8;
				obj.angle = -Math.PI;
				obj.penumbra = 0.3;
				obj.decay = 0.45;
				console.log(obj);
			}
			obj.castShadow = true;
			obj.receiveShadow = true;
		});

		if (gltf.scene.children.length == 1) {
			//console.log("loading with one child");
			appearances[name] = gltf.scene.children[0];
		} else {
			//console.log("loading whole scene.");
			appearances[name] = gltf.scene;
		}
		appearances[name].position.fromArray([0, 0, 0]);
	});
}

// map of entity id -> mesh
const meshes = {};

// add renderer to doc
document.body.appendChild(renderer.domElement);

//let cam_dir_vec = new THREE.Vector3(0.0, 0.0, 0.0);
function render(ents) {
	ents.forEach(({ent, appearance, iso}) => {
		let a = appearances[appearance];

		if (a == undefined) {
			//console.log("no appearance available");
			return;
		}

		if (meshes[ent] == undefined) {
			console.log("added " + appearance, a);
			let mesh = a.clone();
			scene.add(mesh);
			meshes[ent] = mesh;
		}
		
		let mesh = meshes[ent];

		//mesh.quaternion.fromArray(iso.rotation);
		let t = iso.translation;
		mesh.position.fromArray([t[0], 0, -t[1]]);
	});

	//camera.lookAt(cam_dir_vec.fromArray(camera_direction).add(camera.position));

	//composer.render(scene, camera); 
	renderer.render(scene, camera); 
}
