// sample

function main() {
  console.log('Hello, World! 1');
  // -- time-limited-code to="2020-12-31 23:59:59" -- //
    console.log('🧹This code will be removed after 2020-12-31T23:59:59.999Z');
  // -- /time-limited-code -- //

  // -- time-limited-code to="2099-12-31 23:59:59" -- //
    console.log('📌This code will be removed after 2099-12-31T23:59:59.999Z');
  // -- /time-limited-code -- //

  // -- time-limited-code to="2020-12-31 23:59:59" -- //
    console.log('🧹This code will be removed after 2020-12-31T23:59:59.999Z');
  // -- /time-limited-code -- //
  console.log('Hello, World! 2');

  // -- removal-marker name="awesome-feature" unwrap-block -- //
  if (isReleased) {
    console.log('📌This code is unconditionally executed after 2020-12-31T23:59:59.999Z');
    const awesomeFeature = new awesomeFeature()
    awesomeFeature.run();
  }
  // -- /removal-marker -- //
}
