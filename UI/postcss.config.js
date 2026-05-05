/** @type {import('postcss-load-config').Config} */
import cssnano from 'cssnano'
import presetEnv from 'postcss-preset-env'
const config = {
    plugins: [
        presetEnv({
            autoprefixer: true,
            features: {
                'nesting-rules': true,
            },
            browsers: [
				"> 0.2% and not dead"
			],
        }),
        cssnano({
            preset: 'default',
        })
    ]
}

export default config