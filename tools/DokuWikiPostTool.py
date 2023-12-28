# 2021/06/14 Ogasawara Hiroyuki
# vim:ts=4 sw=4 et

import sys
import os
import re
import xml.etree.ElementTree as ET
from xml.dom import minidom
import requests
import base64

#------------------------------------------------------------------------------
# doku_config.txt
#
#  d_server  https://～/wiki
#  d_user    USERNAME
#  d_pass    PASSWORD
#------------------------------------------------------------------------------

# DokuWiki xml-rpc

class DokuUploader:
    def __init__( self, options ):
        self.options= options
        self.server= options['d_server'] + '/lib/exe/xmlrpc.php'
        self.cookies= None
        self.is_debug= options['debug']

    def Element( self, name, text ):
        sub= ET.Element( name )
        sub.text= text
        return  sub

    def Value( self, vtype, value ):
        sub= ET.Element( 'value' )
        vt= ET.Element( vtype )
        vt.text= value
        sub.append( vt )
        return  sub

    def Member( self, name, value ):
        sub= ET.Element( 'member' )
        sub.append( self.Element( 'name', name ) )
        sub.append( value )
        return  sub

    def Array( self, value_list ):
        sub= ET.Element( 'value' )
        array= ET.Element( 'array' )
        data= ET.Element( 'data' )
        sub.append( array )
        array.append( data )
        for value in value_list:
            data.append( value )
        return  sub

    def Struct( self, member_list ):
        sub= ET.Element( 'value' )
        struct= ET.Element( 'struct' )
        sub.append( struct )
        for member in member_list:
            struct.append( member )
        return  sub

    def Param( self, value ):
        sub= ET.Element( 'param' )
        sub.append( value )
        return  sub

    def formatxml( self, xml ):
        xml_str= ET.tostring( xml, 'utf-8' )
        xml_parsed= minidom.parseString( xml_str )
        pretty= re.sub( r'[\t ]+\n', '', xml_parsed.toprettyxml( indent='    ' ) )
        pretty= pretty.replace( '>\n\n\t<', '>\n\t<' )
        return  pretty

    def xml_command( self, command_name, param_list ):
        xml= ET.Element( 'methodCall' )
        xml.append( self.Element( 'methodName', command_name ) )
        params= ET.Element( 'params' )
        for param in param_list:
            params.append( self.Param( param ) )
        # params.append( self.Param( self.Value( 'string', ':' ) ) )
        # params.append( self.Param( self.Array( [ self.Value() .. ] ) ) )
        # params.append( self.Param( self.Struct( [ self.Member( 'sum', self.Value( 'string', 'text' ) ) .. ] ) ) )
        xml.append( params )

        tree= ET.ElementTree( xml )
        xml_str= self.formatxml( xml )
        # print( xml_str )
        # tree.write( 'output.xml', encoding='utf-8', xml_declaration=True )
        return  xml_str

    def xml_parse( self, xml_str ):
        return  ET.fromstring( xml_str )

    def get_param_raw( self, xml_str, param_index ):
        root= self.xml_parse( xml_str )
        if root.tag == 'methodResponse':
            params= root[0]
            param= params[param_index]
            value= param[0]
            return  value[0]
        return  None

    def get_param( self, xml_str, param_index ):
        param= self.get_param_raw( xml_str, param_index )
        if param is not None:
            return  param.text
        return  param

    def send_command( self, command, cookie_save= False ):
        if self.is_debug:
            print( 'post:', self.server )
        res= requests.post( self.server, data=command.encode('utf-8'), verify=False, cookies=self.cookies )
        #body= res.text.decode( self.web_code, 'ignore' )
        body= res.text
        if cookie_save:
            self.cookies= res.cookies
        if self.is_debug:
            print( '--------' )
            print( body )
            print( '--------' )
        return  body

    def f_login( self ):
        params= self.xml_command( 'dokuwiki.login', [
                    self.Value( 'string', self.options['d_user'] ),
                    self.Value( 'string', self.options['d_pass'] ),
                ] )
        result= self.send_command( params, True )
        if self.is_debug:
            print( 'Login=', self.get_param( result, 0 ) )

    def api_put_page( self, page_name, page_data ):
        print( 'PutPage:', page_name )
        params= self.xml_command( 'wiki.putPage', [
                    self.Value( 'string', page_name ),
                    self.Value( 'string', page_data ),
                    self.Struct( [
                            self.Member( 'sum', self.Value( 'string', 'wiki.putPage' ) ),
                            self.Member( 'minor', self.Value( 'boolean', 'True' ) ),
                        ] ),
                ] )
        result= self.send_command( params )
        print( 'PutPage Result:', self.get_param( result, 0 ) )

    def api_get_page( self, page_name ):
        print( 'GetPage:', page_name )
        params= self.xml_command( 'wiki.getPage', [
                    self.Value( 'string', page_name ),
                ] )
        result= self.send_command( params )
        return  self.get_param( result, 0 )

    def api_put_image( self, image_id, image ):
        print( 'PutImage:', image_id )
        params= self.xml_command( 'wiki.putAttachment', [
                    self.Value( 'string', image_id ),
                    self.Value( 'base64', base64.b64encode( image ).decode( 'ascii' ) ),
                ] )
        result= self.send_command( params )
        print( 'PutPage Result:', self.get_param( result, 0 ) )

    def api_get_image_list( self, name_space ):
        print( 'GetImageList:', name_space )
        params= self.xml_command( 'wiki.getAttachments', [
                    self.Value( 'string', name_space ),
                ] )
        result= self.send_command( params )
        array= self.get_param_raw( result, 0 )
        data= array[0]
        image_list= []
        for value in data:
            struct= value[0]
            obj= {}
            for member in struct:
                obj[member[0].text]= member[1][0].text
            image_list.append( obj )
        return  image_list

    def api_get_image( self, image_id ):
        print( 'GetImage:', image_id )
        params= self.xml_command( 'wiki.getAttachment', [
                    self.Value( 'string', image_id ),
                ] )
        result= self.send_command( params )
        text= self.get_param( result, 0 )
        image_data= base64.b64decode( text )
        return  image_data

    #--------------------------------------------------------------------------

    def f_download( self ):
        file_name= self.options['save']
        page_name= self.options['page']
        self.f_login()
        page= self.api_get_page( page_name )
        if file_name:
            with open( file_name, 'w', encoding='utf-8' ) as fo:
                fo.write( page )
        else:
            print( page )

    def f_upload( self ):
        file_name= self.options['file']
        page_name= self.options['page']
        self.f_login()
        if not file_name:
            return
        with open( file_name, 'r', encoding='utf-8' ) as fi:
            page_data= fi.read()
        self.api_put_page( page_name, page_data )

    def f_image_list( self ):
        name_space= self.options['page']
        self.f_login()
        image_list= self.api_get_image_list( name_space )
        for image in image_list:
            print( '%-30s file=%-30s  (%s byte)' % (image['id'], image['file'], image['size'] ) )

    def f_get_image( self ):
        image_id= self.options['page']
        file_name= self.options['save']
        self.f_login()
        image_data= self.api_get_image( image_id )
        if file_name is None:
            file_name= image_id.split( ':' )[-1]
        print( 'save:', file_name )
        with open( file_name, 'wb' ) as fo:
            fo.write( image_data )

    def f_get_image_all( self ):
        name_space= self.options['page']
        self.f_login()
        image_list= self.api_get_image_list( name_space )
        for image in image_list:
            print( '%-30s file=%-30s  (%s byte)' % (image['id'], image['file'], image['size'] ) )
            image_data= self.api_get_image( image['id'] )
            file_name= image['file']
            print( 'save:', file_name )
            with open( file_name, 'wb' ) as fo:
                fo.write( image_data )

    def f_put_image( self ):
        name_space= self.options['page']
        file_name= self.options['file']
        self.f_login()
        print( 'load:', file_name )
        with open( file_name, 'rb' ) as fi:
            image_data= fi.read()
        image_id= name_space + ':' + os.path.basename( file_name )
        self.api_put_image( image_id, image_data )


#------------------------------------------------------------------------------

def load_config( config_file, options ):
    if not os.path.exists( config_file ):
        return
    with open( config_file, 'r', encoding='utf-8' ) as fi:
        for line in fi:
            line= line.strip()
            if line == '' or line[0] == '#':
                continue
            param_list= line.split()
            options[param_list[0]]= param_list[1]

def usage():
    print( 'DokuWikiPostTool v1.00 Hiroyuki Ogasawara' )
    print( 'usage: DokuWikiPostTool.py [<options>..]' )
    print( 'options:' )
    print( ' --config <config.txt>' )
    print( ' --page <page>' )
    print( ' --file <file.txt>' )
    print( ' --save <file.txt>' )
    print( ' --server <server>' )
    print( ' --download              (--page PAGE --save FILE)' )
    print( ' --upload                (--page PAGE --file FILE)' )
    print( ' --image_list            (--page NAMESPACE)' )
    print( ' --get_image             (--page IMAGEID [--save FILE])' )
    print( ' --put_image             (--page NAMESPACE --file FILE)' )
    print( ' --get_image_all         (--page NAMESPACE)' )
    print( ' --login' )
    print( ' --debug' )
    print( 'ex. --download --page test:start --save page.txt' )
    print( 'ex. --upload --page test:start --file page.txt' )
    print( 'ex. --get_image --page test:data.jpeg' )
    print( 'ex. --put_image --page test --file data.jpeg' )
    sys.exit( 1 )


def main( argv ):
    options= {
            'config'   : 'doku_config.txt',
            'd_server' : 'http://localhost/wiki',
            'func' : None,
            'page' : None,
            'file' : None,
            'save' : None,
            'debug': False,
        }
    acount= len(argv)
    ai= 1
    while ai< acount:
        arg= argv[ai]
        if arg.startswith( '-' ):
            if arg == '--config':
                if ai+1 < acount:
                    ai+= 1
                    options['config']= argv[ai]
            elif arg == '--page':
                if ai+1 < acount:
                    ai+= 1
                    options['page']= argv[ai]
            elif arg == '--file':
                if ai+1 < acount:
                    ai+= 1
                    options['file']= argv[ai]
            elif arg == '--save':
                if ai+1 < acount:
                    ai+= 1
                    options['save']= argv[ai]
            elif arg == '--server':
                if ai+1 < acount:
                    ai+= 1
                    options['d_server']= argv[ai]
            elif arg == '--debug':
                options['debug']= True
            elif arg == '--login':
                options['func']= 'f_login'
            elif arg == '--download':
                options['func']= 'f_download'
            elif arg == '--upload':
                options['func']= 'f_upload'
            elif arg == '--image_list':
                options['func']= 'f_image_list'
            elif arg == '--get_image':
                options['func']= 'f_get_image'
            elif arg == '--put_image':
                options['func']= 'f_put_image'
            elif arg == '--get_image_all':
                options['func']= 'f_get_image_all'
            else:
                usage()
        else:
            pass
        ai+= 1

    if options['func']:
        load_config( options['config'], options )
        uploader= DokuUploader( options )
        func= options['func']
        if hasattr( uploader, func ):
            getattr( uploader, func )()
        else:
            print( 'Error', func )
    else:
        usage()

    return  0

if __name__=='__main__':
    sys.exit( main( sys.argv ) )


