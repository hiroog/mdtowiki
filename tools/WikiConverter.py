# 2023/12/26 Ogasawara Hiroyuki
# vim:ts=4 sw=4 et:

import sys
import os
import zipfile
import DokuWikiPostTool

#  notion/...zip
#  dokuwiki_map.txt
#    post  document:tools:converter  8cf1.doku.txt


class WikiConverter:
    def __init__( self, options ):
        self.options= options
        self.load_dokuwiki_map()

    def find_mdfile( self, folder_root ):
        for file_name in os.listdir( folder_root ):
            base_name,ext= os.path.splitext( file_name )
            if ext == '.md':
                md_path= os.path.join( folder_root, file_name )
                print( 'found: ', md_path )
                return  md_path
        return  None

    def load_dokuwiki_map( self ):
        file_name= 'dokuwiki_map.txt'
        post_list= []
        if os.path.exists( file_name ):
            with open( file_name, 'r', encoding='utf-8' ) as fi:
                for line in fi:
                    line= line.strip()
                    if line == '' or line.startswith( '#' ):
                        continue
                    params= line.split()
                    if params[0] == 'post':
                        print( 'LOAD PostMap', params )
                        post_list.append( { 'page':params[1], 'file':params[2] } )
        self.post_list= post_list

    def auto_post( self, file_doku ):
        for post in self.post_list:
            if file_doku.endswith( post['file'] ):
                print( 'MATCH', post )
                DokuWikiPostTool.main( [ '',
                            '--upload', '--page', post['page'], '--file', file_doku
                        ] )
                break;

    def convert_file( self, file_name, output_file ):
        print( 'load:', file_name )
        input_file= file_name
        output_file_doku= output_file + '.doku.txt'
        output_file_conf= output_file + '.conf.txt'
        command= 'cargo run -- -lmd "%s" -sdoku "-o%s" -sconf "-o%s"' % (input_file, output_file_doku, output_file_conf)
        print( 'save:', output_file )
        os.system( command )
        self.auto_post( output_file_doku )

    def decode_notion( self, folder_root ):
        for file_name in os.listdir( folder_root ):
            base_name,ext= os.path.splitext( file_name )
            if ext == '.zip':
                zip_dir_path= os.path.join( folder_root, base_name )
                if not os.path.exists( zip_dir_path ):
                    with zipfile.ZipFile( zip_dir_path + '.zip' ) as zi:
                        zi.extractall( zip_dir_path )
                        print( 'extract: ', zip_dir_path+'.zip' )
                md_path= self.find_mdfile( zip_dir_path )
                if md_path:
                    base,name= os.path.split( md_path )
                    output_name= os.path.join( folder_root, name.replace( '.md', '' ) )
                    self.convert_file( md_path, output_name )

    def f_convert_zip( self ):
        self.decode_notion( 'notion' )

    def f_convert_file( self ):
        self.convert_file( self.options['file'], self.options['save'] )


def usage():
    print( 'WikiConverter v1.00 Hiroyuki Ogasawara' )
    print( 'usage: WikiConverter [options]' )
    print( '  --notion' )
    print( '  --src <file>' )
    print( '  --save <file>     (default output)' )
    print( '  --mdfilter' )
    print( 'ex. --src src.md' )
    print( 'ex. --notion' )
    sys.exit( 1 )


def main( argv ):
    options= {
            'file': None,
            'save': 'output',
            'mdfilter': False,
            'func': None
        }
    acount= len(argv)
    ai= 1
    while ai< acount:
        arg= argv[ai]
        if arg == '--src':
            if ai+1 < acount:
                ai+= 1
                options['file']= argv[ai]
            options['func']= 'f_convert_file'
        elif arg == '--save':
            if ai+1 < acount:
                ai+= 1
                options['save']= argv[ai]
        elif arg == '--mdfilter':
            options['mdfilter']= True
        elif arg == '--notion':
            options['func']= 'f_convert_zip'
        else:
            usage()
        ai+= 1
    if not options['func']:
        usage()
    no= WikiConverter( options )
    func= options['func']
    if hasattr( no, func ):
        getattr( no, func )()
    return  0


if __name__=='__main__':
    sys.exit( main( sys.argv ) )

