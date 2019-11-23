// remove padding from sides of site, to make room for canvas
document.body.style.margin = "0px";
document.body.style.padding = "0px";
document.body.style.overflow = "hidden";

let renderer = new THREE.WebGLRenderer( { antialias: true } );
renderer.setSize(window.innerWidth, window.innerHeight);

// camera setup
let camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.01, 100);
camera.position.y = 40;
camera.position.x = 70;
camera.position.z = -50;
camera.lookAt(40, 0, -50);

// where all of the things to render are stored
let scene = new THREE.Scene();

var light = new THREE.AmbientLight(0xCDDEFF)
scene.add(light);

// map of appearance name -> mesh
let appearances = {
	RockHole: [new THREE.BoxGeometry(3.8, 0.2, 3.8), new THREE.MeshNormalMaterial()],
	Rock: [new THREE.BoxGeometry(3.8, 0.2, 3.8), new THREE.MeshLambertMaterial()]
};

// map of entity id -> mesh
let meshes = {};

// add renderer to doc
document.body.appendChild(renderer.domElement);

let cam_dir_vec = new THREE.Vector3(0.0, 0.0, 0.0);
function render(ents) {
	ents.forEach(({ent, appearance, iso}) => {
		if (meshes[ent] == undefined) {
			let [geometry, material] = appearances[appearance];
			let mesh = new THREE.Mesh(geometry, material);
			scene.add(mesh);
			meshes[ent] = mesh;
		}
		
		let mesh = meshes[ent];

		//mesh.quaternion.fromArray(iso.rotation);
		let t = iso.translation;
		mesh.position.fromArray([t[0], 0, -t[1]]);
	});

	//camera.lookAt(cam_dir_vec.fromArray(camera_direction).add(camera.position));

	renderer.render(scene, camera); 
}
