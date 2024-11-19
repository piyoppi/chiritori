async function main() {
  console.log('Hello, World! 1 [0]');
  /* < time-limited to="2020-12-31 23:59:59" > */
    console.log('🧹This code will be removed after 2020-12-31T23:59:59.999Z [1]');
  /* < /time-limited > */

  /* < time-limited to="2099-12-31 23:59:59" > */
    console.log('📌This code will be removed after 2099-12-31T23:59:59.999Z [2]');
  /* < /time-limited > */

  /* < time-limited to="2020-12-31 23:59:59" > */
    console.log('🧹This code will be removed after 2020-12-31T23:59:59.999Z [3]');
  /* < /time-limited > */

  /* =============== Nested tags =============== */

  /* < time-limited to="2020-12-31 23:59:59" > */
    console.log('🧹This code will be removed after 2020-12-31T23:59:59.999Z [4]');
    /* < time-limited to="2099-12-31 23:59:59" > */
      console.log('📌This code will be removed after 2099-12-31T23:59:59.999Z [5]');
    /* < /time-limited > */
  /* < /time-limited > */

  /* < time-limited to="2099-12-31 23:59:59" > */
    console.log('📌This code will be removed after 2099-12-31T23:59:59.999Z [6]');
    /* < time-limited to="2020-12-31 23:59:59" > */
      console.log('🧹This code will be removed after 2020-12-31T23:59:59.999Z [7]');
    /* < /time-limited > */
  /* < /time-limited > */

  const isReleased = await fetch('https://example.test/features/awesome-feature')

  /* =============== Unwrap Block =============== */

  /* < time-limited to="2020-12-31 23:59:59" unwrap-block > */
  if (isReleased) {
    console.log('📌This code is unconditionally executed after 2020-12-31T23:59:59.999Z [8]');
    const awesomeFeature = new awesomeFeature()
    awesomeFeature.run();
  }
  /* < /time-limited > */

  for (let i = 0; i < 10; i++) {
    /* < time-limited to="2020-12-31 23:59:59" unwrap-block > */
    if (isReleased) {
      console.log('📌This code is unconditionally executed after 2020-12-31T23:59:59.999Z [9]');
      awesomeFeature.add(i);
    }
    /* < /time-limited > */
  }

/* < time-limited to="2020-12-31 23:59:59" unwrap-block > */
  if (isReleased) {
    console.log('📌This code is unconditionally executed after 2020-12-31T23:59:59.999Z [10]');
  }
/* < /time-limited > */

  /* unwrap-block requires a removal target immediately after the start tag and immediately before the end tag. */
  /* < time-limited to="2020-12-31 23:59:59" unwrap-block > */
  /* < /time-limited > */
  /* < time-limited to="2020-12-31 23:59:59" unwrap-block > */
  foo()
  /* < /time-limited > */

  /* If the indentation of the marker is greater than the indentation of the block, it cannot be removed. */
        /* < time-limited to="2020-12-31 23:59:59" unwrap-block > */
  if (isReleased) {
    console.log('📌This code is unconditionally executed after 2020-12-31T23:59:59.999Z [11]');
  }
        /* < /time-limited > */

  /* unwrap-block requires a line break immediately after the start tag and immediately before the end tag. */
  /* < time-limited to="2020-12-31 23:59:59" unwrap-block > */ foo() /* < /time-limited > */
  /* < time-limited to="2020-12-31 23:59:59" unwrap-block > *//* < /time-limited > */
  console.log('Hello, World! 2 [12]');

  /* =============== comment attribute =============== */

  /* < time-limited to="2001-12-31 23:59:59"
   * c="You can write your comments here."
   * > */
    console.log('[13]');
  /* < /time-limited > */

  /* < time-limited to="2020-12-31 23:59:59" unwrap-block
   * c="You can write your comments here."
   * > */
  if (isReleased) {
    console.log('[14]');
  }
  /* < /time-limited > */
}
