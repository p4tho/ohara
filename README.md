# ohara
Create a list of arxiv URLs to create .epub files for your ereader.

## Installation
1. Clone the repository
```
git clone https://github.com/p4tho/ohara.git
cd ohara
```

## Usage
1. Create a .txt file to list arxiv papers you want to create .epub files for
```
https://arxiv.org/abs/2607.04379
https://arxiv.org/abs/2607.04055
https://arxiv.org/abs/2607.03288
https://arxiv.org/abs/2607.02875
https://arxiv.org/abs/2607.06141
```

2. Run the program with desired .txt and output directories for .epub files
```
go run . arxiv_urls.txt -o output_dir/
```

3. Enjoy your .epub files