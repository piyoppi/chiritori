/*
 * Tab indent
 */
async function main() {
	/* < time-limited to="2020-12-31 23:59:59" > */
	console.log('foo')
	/* < /time-limited > */

	console.log('bar')

	/* < time-limited to="2020-12-31 23:59:59" unwrap-block > */
	if (isReleased) {
		console.log('📌This code is unconditionally executed after 2020-12-31T23:59:59.999Z [8]');
		const awesomeFeature = new awesomeFeature()
		awesomeFeature.run();
	}
	/* < /time-limited > */
}
