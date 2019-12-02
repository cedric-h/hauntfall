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
const cam_offset = new THREE.Vector3(15, 20, 15);
cam_offset.multiplyScalar(0.75);
camera.position.copy(cam_offset);
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

// list of appearance names
let appearance_names;
// map of appearance_names index -> Mesh
const appearances = {};
const loader = new THREE.GLTFLoader().setPath("assets/");
function load_assets(a_ns) {
	appearance_names = a_ns;
	appearance_names.forEach((name, index) => {
		loader.load(name + ".glb", (gltf) => {
			//console.log("loaded one!", gltf);
			console.log("loading:", name);

			gltf.scene.traverse((obj) => {
				if (obj.type == "SpotLight") {
					obj.intensity /= 225;
					obj.distance = 8;
					obj.angle = -Math.PI;
					obj.penumbra = 0.3;
					obj.decay = 0.45;
					//console.log(obj);
				}
				obj.castShadow = true;
				obj.receiveShadow = true;
			});

			if (gltf.scene.children.length == 1) {
				//console.log("loading with one child");
				appearances[index] = gltf.scene.children[0];
			} else {
				//console.log("loading whole scene.");
				appearances[index] = gltf.scene;
			}
			appearances[index].position.fromArray([0, 0, 0]);

			// give every entity that was waiting for this asset to load
			// its new appearance, now that it's loaded.
			//console.log('prepare: ' + index);
			for (const loading_ent in loading_appearance_indexes) {
				//console.log(loading_appearance_indexes[loading_ent] + " has been queued to load");
				if (loading_appearance_indexes[loading_ent] == index) {
					//console.log("now that " + name + " is loaded, we're assigning it")
					set_appearance({ent: loading_ent, appearance_index: index});
					delete loading_appearance_indexes[loading_ent];
				}
			}
		});
	});
}

// map of entity id -> mesh
const meshes = {};

// map of entity id -> appearance name,
// if that appearance name is still loading
const loading_appearance_indexes = {};

// add renderer to doc
document.body.appendChild(renderer.domElement);

function set_appearance({ent, appearance_index}) {
	if (!(appearance_index in appearances)) {
		// if they want a mesh that isn't loaded yet, we'll store the name of it.
		//console.log("queing " + appearance_index + " mesh to load");
		loading_appearance_indexes[ent] = appearance_index;
		return;
	}

	//console.log("actually making mesh");
	let mesh = appearances[appearance_index].clone();
	scene.add(mesh);
	meshes[ent] = mesh;
}

function clear_appearance(ent) {
	if (meshes[ent] != undefined) {
		scene.remove(meshes[ent]);
		delete meshes[ent];
	} else if (loading_appearance_indexes[ent] != undefined) {
		delete loading_appearance_indexes[ent];
	} else {
		console.error("limbo mesh? not loading or loaded", ent);
	}
}

const cam = new THREE.Vector3(0.0, 0.0, 0.0);
function render(ents, player) {
	ents.forEach(({ent, rot, iso}) => {
		let mesh = meshes[ent];

		if (mesh != undefined) {
			//mesh.quaternion.fromArray(iso.rotation);
			let t = iso.translation;
			mesh.position.fromArray([t[0], 0, -t[1]]);
			mesh.rotation.y = rot;
		} else {
			console.log("We can't position a mesh because the mesh doesn't exist");
		}
	});

	camera.position.fromArray([player.vec[0], 0, -player.vec[1]]);
	camera.position.add(cam_offset);

	//composer.render(scene, camera); 
	renderer.render(scene, camera); 
}
