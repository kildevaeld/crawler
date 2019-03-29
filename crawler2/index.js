


module.exports = function (pack) {
    //process.stdout.write("\nHello, World " + pack.content + '\n');
    // process.stdout.write(pack.content)
    // process.stdout.write('\n\n');
    return [
        $ok(pack),
        // $work($package("", "http://loppen.dk/concert/id"), [
        //     worktype.Http({ method: 'GET' }),
        //     worktype.Duktape({ script: './concert.js' })
        // ])
    ]
}