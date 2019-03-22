


module.exports = function (pack) {
    //process.stdout.write("\nHello, World " + pack.content + '\n');
    process.stdout.write(pack.name)
    return [
        $ok(pack),
        $work([
            worktype.Http({ method: 'GET' }),
            worktype.Duktape({ script: './concert.js' })
        ])
    ]
}