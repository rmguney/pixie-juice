// Test loader imports
const { GLTFLoader } = require('three-stdlib');

const loader = new GLTFLoader();
console.log('GLTFLoader methods:', Object.getOwnPropertyNames(Object.getPrototypeOf(loader)));
console.log('Has setLoadingManager:', typeof loader.setLoadingManager);
console.log('Has parse:', typeof loader.parse);
