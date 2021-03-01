console.log("File loaded");
const fileSelector = document.getElementById('file-selector');
fileSelector.addEventListener('change', function() {
    var fr=new FileReader();
    fr.onload=function(){ 
        // fetch("/upload", {
        //     method: "POST",
        //     headers: {
        //         'Content-Type': 'text/plain'
        //         // 'Content-Type': 'application/x-www-form-urlencoded',
        //     },
        //     body: fr.result
        // }).then(res => {
        //     console.log("Request complete! response:", res);
        // }); 
        console.log(fr.result)
    } 
              
    fr.readAsText(this.files[0]); 
});
