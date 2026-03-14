async function main() {
  console.log('Hello, World!');

  /* < time-limited from="2020-01-01 00:00:00" > */
  console.log('cleanup-target: past from, no to [1]');
  /* < /time-limited > */

  /* < time-limited from="2099-01-01 00:00:00" > */
  console.log('no-cleanup: future from, no to [2]');
  /* < /time-limited > */

  /* < time-limited to="2020-12-31 23:59:59" > */
  console.log('removal-target: past to [3]');
  /* < /time-limited > */

  /* < time-limited from="2020-01-01 00:00:00" to="2099-12-31 23:59:59" > */
  console.log('no-cleanup: has both from and to [4]');
  /* < /time-limited > */

  console.log('End');
}
